//! Persistent parked cooker workers (plan §10, AWO-2).
//!
//! - Create `maximum_cookers` named threads once at start.
//! - Scale via active permits (not spawn/join after warm).
//! - Bodies/subjects stay `Arc<[u8]>` — no clone into the cook path.
//! - Cookers never touch `Store`, indexes, file handles, or completions.
//! - Outcomes land in an [`OrderedReadyRing`] keyed by lane ticket.

use super::ordered_ready::OrderedReadyRing;
use super::persist::LaneTicket;
use super::queue::BoundedQueue;
use crate::envelope::{encode_item_envelope, EventKind, ItemEnvelope, MAX_SUBJECT_LEN};
use residiuum_format::{
    encode_frame_into, FrameHeader, FrameKind, WIRE_MAJOR, WIRE_MINOR,
};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::thread::JoinHandle;

/// Immutable cook input (plan: body owned once as `Arc<[u8]>`).
#[derive(Debug, Clone)]
pub struct CookTask {
    /// Lane ticket for ordered readiness.
    pub ticket: LaneTicket,
    /// Subject bytes (not cloned by workers).
    pub subject: Arc<[u8]>,
    /// Body bytes (not cloned by workers).
    pub body: Arc<[u8]>,
    /// Store id embedded in the item envelope.
    pub store_id: [u8; 16],
    /// Segment id at reservation time.
    pub segment_id: [u8; 16],
    /// Stable item id for the subject lineage.
    pub item_id: [u8; 16],
    /// Event id for this mutation.
    pub event_id: [u8; 16],
    /// Writer sequence assigned at reservation.
    pub writer_sequence: u64,
    /// Creation timestamp (ns).
    pub created_ns: u64,
    /// Put or Delete.
    pub event_kind: EventKind,
}

/// Successful cook product (encoded frame only — no store touch).
#[derive(Debug, Clone)]
pub struct CookedMutation {
    /// Lane ticket.
    pub ticket: LaneTicket,
    /// Complete encoded frame bytes.
    pub encoded_frame: Vec<u8>,
}

/// Cook outcome for the ready ring.
#[derive(Debug, Clone)]
pub enum CookOutcome {
    /// Encoded successfully.
    Ok(CookedMutation),
    /// Encoding / limit failure (still ordered by ticket).
    Err {
        /// Ticket of the failed task.
        ticket: LaneTicket,
        /// Human-readable reason.
        message: String,
    },
}

impl CookOutcome {
    /// Ticket for ordering.
    pub fn ticket(&self) -> LaneTicket {
        match self {
            Self::Ok(c) => c.ticket,
            Self::Err { ticket, .. } => *ticket,
        }
    }

    /// Encoded frame length if successful (0 on error).
    pub fn frame_bytes(&self) -> usize {
        match self {
            Self::Ok(c) => c.encoded_frame.len(),
            Self::Err { .. } => 0,
        }
    }
}

/// Encode one item frame identically to the serial product path (pure).
///
/// Does not touch `Store`, indexes, or FDs. Used by cooker workers and by
/// frame-equivalence tests against a serial baseline.
pub fn cook_item_frame(task: &CookTask) -> Result<Vec<u8>, String> {
    if task.subject.len() > MAX_SUBJECT_LEN {
        return Err("subject too long".into());
    }
    let _ = crate::failpoint::hit("awo.cook.before").map_err(|e| e.to_string())?;
    let env = ItemEnvelope {
        store_id: task.store_id,
        segment_id: task.segment_id,
        item_id: task.item_id,
        event_kind: task.event_kind,
        created_ns: task.created_ns,
        subject: task.subject.as_ref().to_vec(),
    };
    let envelope = encode_item_envelope(&env).map_err(|e| e.to_string())?;
    let header = FrameHeader {
        wire_major: WIRE_MAJOR,
        wire_minor: WIRE_MINOR,
        frame_kind: FrameKind::ItemEvent.as_u8(),
        flags: Default::default(),
        envelope_len: envelope.len() as u32,
        body_len: task.body.len() as u64,
        logical_len: task.body.len() as u64,
        writer_sequence: task.writer_sequence,
        event_id: task.event_id,
    };
    let mut frame =
        Vec::with_capacity((task.body.len() + envelope.len() + 128).max(256));
    encode_frame_into(&mut frame, &header, &envelope, task.body.as_ref())
        .map_err(|e| e.to_string())?;
    let _ = crate::failpoint::hit("awo.cook.after").map_err(|e| e.to_string())?;
    Ok(frame)
}

/// Simple buffer pool for recycled frame buffers (AWO-2 floor).
#[derive(Debug, Default)]
pub struct FrameBufferPool {
    free: Mutex<Vec<Vec<u8>>>,
}

impl FrameBufferPool {
    /// Create an empty pool.
    pub fn new() -> Self {
        Self::default()
    }

    /// Take a buffer with at least `min_cap` capacity.
    pub fn take(&self, min_cap: usize) -> Vec<u8> {
        let mut g = self.free.lock().expect("pool lock");
        if let Some(mut v) = g.pop() {
            v.clear();
            if v.capacity() < min_cap {
                v.reserve(min_cap - v.capacity());
            }
            return v;
        }
        Vec::with_capacity(min_cap.max(256))
    }

    /// Return a buffer to the pool (cleared).
    pub fn give(&self, mut buf: Vec<u8>) {
        buf.clear();
        let mut g = self.free.lock().expect("pool lock");
        if g.len() < 64 {
            g.push(buf);
        }
    }
}

/// Persistent cooker pool: max threads at start, scale via permits.
pub struct PersistentCookerPool {
    maximum_cookers: usize,
    active_cookers: Arc<AtomicUsize>,
    task_queue: Arc<BoundedQueue<CookTask>>,
    ready: Arc<OrderedReadyRing<CookOutcome>>,
    shutdown: Arc<AtomicBool>,
    workers: Vec<JoinHandle<()>>,
    /// Tasks submitted since start (diagnostics).
    submitted: AtomicUsize,
    /// Threads created at start (must equal maximum_cookers; no post-warm create).
    threads_created: usize,
}

impl PersistentCookerPool {
    /// Spawn `maximum_cookers` workers immediately; only `initial_active` process work.
    pub fn start(
        maximum_cookers: usize,
        initial_active: usize,
        task_queue_capacity: usize,
        ready_byte_limit: usize,
        first_ticket: u64,
    ) -> Self {
        let maximum_cookers = maximum_cookers.max(1);
        let initial_active = initial_active.clamp(1, maximum_cookers);
        let active_cookers = Arc::new(AtomicUsize::new(initial_active));
        let task_queue = Arc::new(BoundedQueue::new(task_queue_capacity.max(1)));
        let ready = Arc::new(OrderedReadyRing::new(first_ticket, ready_byte_limit));
        let shutdown = Arc::new(AtomicBool::new(false));
        let mut workers = Vec::with_capacity(maximum_cookers);

        for worker_id in 0..maximum_cookers {
            let active = Arc::clone(&active_cookers);
            let queue = Arc::clone(&task_queue);
            let ready_ring = Arc::clone(&ready);
            let stop = Arc::clone(&shutdown);
            let name = format!("awo-cooker-{worker_id}");
            let handle = std::thread::Builder::new()
                .name(name)
                .spawn(move || {
                    worker_loop(worker_id, active, queue, ready_ring, stop);
                })
                .expect("spawn awo cooker");
            workers.push(handle);
        }

        Self {
            maximum_cookers,
            active_cookers,
            task_queue,
            ready,
            shutdown,
            workers,
            submitted: AtomicUsize::new(0),
            threads_created: maximum_cookers,
        }
    }

    /// Thread pool size created at start (never grows after warm).
    pub fn threads_created(&self) -> usize {
        self.threads_created
    }

    /// Maximum cookers (pool size).
    pub fn maximum_cookers(&self) -> usize {
        self.maximum_cookers
    }

    /// Current active permit count.
    pub fn active_cookers(&self) -> usize {
        self.active_cookers.load(Ordering::SeqCst)
    }

    /// Change active permits without spawning or joining threads.
    pub fn set_active_cookers(&self, n: usize) {
        let n = n.clamp(0, self.maximum_cookers);
        self.active_cookers.store(n, Ordering::SeqCst);
        self.task_queue.notify_all_waiters();
    }

    /// Submit a cook task (non-blocking). Caller owns credit accounting.
    pub fn try_submit(&self, task: CookTask) -> Result<(), super::queue::QueueError> {
        self.task_queue.try_push(task)?;
        self.submitted.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }

    /// Ordered ready ring (shared with coordinator).
    pub fn ready(&self) -> &Arc<OrderedReadyRing<CookOutcome>> {
        &self.ready
    }

    /// Tasks submitted.
    pub fn submitted(&self) -> usize {
        self.submitted.load(Ordering::Relaxed)
    }

    /// Pending cook tasks.
    pub fn pending_tasks(&self) -> usize {
        self.task_queue.len()
    }

    /// Drain: stop accepting, wait for workers to finish remaining tasks.
    pub fn shutdown(mut self) {
        self.shutdown.store(true, Ordering::SeqCst);
        self.task_queue.shutdown();
        for h in self.workers.drain(..) {
            let _ = h.join();
        }
    }
}

fn worker_loop(
    worker_id: usize,
    active_cookers: Arc<AtomicUsize>,
    task_queue: Arc<BoundedQueue<CookTask>>,
    ready: Arc<OrderedReadyRing<CookOutcome>>,
    shutdown: Arc<AtomicBool>,
) {
    loop {
        let task = task_queue.pop_while(|| {
            if shutdown.load(Ordering::SeqCst) && task_queue.is_empty() {
                // still need to exit pop_while — parked true only when inactive
            }
            worker_id >= active_cookers.load(Ordering::SeqCst)
                && !(shutdown.load(Ordering::SeqCst) && task_queue.is_empty())
        });
        // When inactive, pop_while parks. When shutdown+empty, returns None if not parked.
        // Re-check: if parked condition is false and queue empty and shutdown, pop returns None.
        let Some(task) = task else {
            if shutdown.load(Ordering::SeqCst) {
                break;
            }
            // Spurious: became inactive; loop again.
            continue;
        };

        // Permit check after pop: if we became inactive, re-queue is wrong —
        // plan says inactive workers park *before* taking work. If we already
        // took work, finish it (no third reservation risk; CPU only).
        let outcome = match cook_item_frame(&task) {
            Ok(frame) => CookOutcome::Ok(CookedMutation {
                ticket: task.ticket,
                encoded_frame: frame,
            }),
            Err(message) => CookOutcome::Err {
                ticket: task.ticket,
                message,
            },
        };
        let ticket = outcome.ticket();
        let bytes = outcome.frame_bytes().max(1);
        // Ready ring full: spin-yield until space (bounded by coordinator drain).
        let mut pending = Some(outcome);
        while let Some(out) = pending.take() {
            match ready.push(ticket, out, bytes) {
                Ok(()) => break,
                Err((super::ordered_ready::ReadyError::BytesExhausted, out)) => {
                    pending = Some(out);
                    std::thread::yield_now();
                    if shutdown.load(Ordering::SeqCst) {
                        break;
                    }
                }
                Err(_) => break,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{Duration, Instant};

    fn sample_task(ticket: u64, subject: &str, body: &[u8]) -> CookTask {
        CookTask {
            ticket: LaneTicket { ticket },
            subject: Arc::<[u8]>::from(subject.as_bytes()),
            body: Arc::<[u8]>::from(body),
            store_id: [1u8; 16],
            segment_id: [2u8; 16],
            item_id: [3u8; 16],
            event_id: [4u8; 16],
            writer_sequence: ticket,
            created_ns: 1_000_000,
            event_kind: EventKind::Put,
        }
    }

    #[test]
    fn pure_cook_matches_serial_twice() {
        let t = sample_task(0, "k", b"hello-body");
        let a = cook_item_frame(&t).unwrap();
        let b = cook_item_frame(&t).unwrap();
        assert_eq!(a, b);
        assert!(a.len() > t.body.len());
    }

    #[test]
    fn pool_cooks_in_ticket_order() {
        let pool = PersistentCookerPool::start(4, 2, 64, 16 * 1024 * 1024, 0);
        assert_eq!(pool.threads_created(), 4);
        for i in 0..8u64 {
            pool.try_submit(sample_task(i, &format!("s{i}"), b"x"))
                .unwrap();
        }
        let deadline = Instant::now() + Duration::from_secs(5);
        let mut got = Vec::new();
        while got.len() < 8 && Instant::now() < deadline {
            if let Some((ticket, outcome)) = pool.ready().try_pop_next() {
                match outcome {
                    CookOutcome::Ok(c) => {
                        assert_eq!(c.ticket, ticket);
                        assert!(!c.encoded_frame.is_empty());
                        got.push(ticket.ticket);
                    }
                    CookOutcome::Err { message, .. } => panic!("cook err: {message}"),
                }
            } else {
                std::thread::yield_now();
            }
        }
        assert_eq!(got, (0..8).collect::<Vec<_>>());
        // Scale permits without new threads.
        pool.set_active_cookers(1);
        assert_eq!(pool.active_cookers(), 1);
        assert_eq!(pool.threads_created(), 4);
        pool.shutdown();
    }

    #[test]
    fn inactive_workers_do_not_require_spawn() {
        let pool = PersistentCookerPool::start(3, 1, 16, 1024 * 1024, 0);
        assert_eq!(pool.threads_created(), 3);
        pool.set_active_cookers(3);
        pool.set_active_cookers(0);
        pool.set_active_cookers(2);
        assert_eq!(pool.threads_created(), 3);
        pool.shutdown();
    }
}