//! Background seal / checkpoint pipeline (DEF-096 Axis A).
//!
//! Foreground put path does an **O(1) rotate** (rename active → pending, open a
//! new active). A worker thread finalizes seals: summary append (offset-
//! preserving), sealed image write, BLAKE3, Hydra, and Chimera. Derived index
//! checkpoints can also run on the worker so `persist_index_cache` fsyncs leave
//! the put acknowledgement path.
//!
//! Correctness:
//! - Pending files are authoritative until sealed (included in open recovery).
//! - Frame offsets are preserved (`ActiveSegment::resume_unsealed` + seal).
//! - Bounded inflight seals provide backpressure when workers lag.
//! - Explicit [`crate::Store::seal_active`] still runs the synchronous path and
//!   drains the pipeline first (tests / failpoints).

use crate::error::StoreError;
use crate::hydra::{
    hydra_index_path, records_from_segment_bytes, write_hydra_index, HydraBuildOptions,
};
use crate::index::PrimaryIndex;
use crate::index_cache::{write_primary_index_frontier, IndexFrontier};
use crate::layout::{list_residiuum_files, segment_id_from_filename, StorePaths};
use residiuum_format::{
    decode_descriptor_body, scan_forward, ActiveSegment, FrameKind, SafetyLimits, SegmentId,
};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::PathBuf;
use std::sync::mpsc::{self, Receiver, RecvTimeoutError, Sender, TryRecvError};
use std::thread::{self, JoinHandle};
use std::time::Duration;

/// Default bound on seals in flight (dual-slot: live active + pending finalize).
pub const DEFAULT_MAX_PENDING_SEALS: usize = 2;

/// Job submitted to the lifecycle worker.
pub enum LifecycleJob {
    /// Finalize a rotated active segment sitting under `active/pending/`.
    FinalizeSeal {
        /// Store identity.
        store_id: [u8; 16],
        /// Segment id (filename stem).
        segment_id: [u8; 16],
        /// Path of the unsealed pending file.
        pending_path: PathBuf,
        /// Destination sealed path under `segments/`.
        sealed_path: PathBuf,
        /// Safety limits used while writing.
        limits: SafetyLimits,
        /// Store root paths (Hydra/Chimera layout).
        paths: StorePaths,
        /// When true, `sync_all` sealed image + parent dir (Durable ack path).
        /// When false, write+rename only (Buffered-only segments; CSQ-ACK-004).
        require_fsync: bool,
    },
    /// Write a primary-index frontier checkpoint (derived only).
    Checkpoint {
        /// Destination cache path.
        cache_path: PathBuf,
        /// Store identity.
        store_id: [u8; 16],
        /// Frontier metadata.
        frontier: IndexFrontier,
        /// Durable index snapshot (locator-first).
        index: PrimaryIndex,
    },
    /// Stop the worker after draining queued jobs.
    Shutdown,
}

/// Result posted by the worker after a job completes.
#[derive(Debug)]
pub enum LifecycleResult {
    /// Seal finalized; sealed file is durable.
    SealDone {
        /// Segment id.
        segment_id: [u8; 16],
        /// BLAKE3 of sealed image.
        content_hash: [u8; 32],
        /// Sealed byte length.
        size: u64,
        /// Sealed image bytes (for catalog summary; dropped after apply).
        sealed_bytes: Vec<u8>,
    },
    /// Checkpoint written (or best-effort failed — see `ok`).
    CheckpointDone {
        /// Whether the write succeeded.
        ok: bool,
    },
    /// Seal finalize failed (pending file may remain for recovery).
    SealFailed {
        /// Segment id.
        segment_id: [u8; 16],
        /// Error text.
        error: String,
    },
}

/// Background lifecycle worker handle owned by a writer `Store`.
pub struct SealPipeline {
    job_tx: Sender<LifecycleJob>,
    result_rx: Receiver<LifecycleResult>,
    join: Option<JoinHandle<()>>,
    /// Jobs submitted and not yet applied via result_rx.
    pub inflight_seals: usize,
    /// Max seals allowed in flight before put backpressure.
    pub max_pending_seals: usize,
}

impl SealPipeline {
    /// Spawn the worker thread.
    pub fn start() -> Self {
        let (job_tx, job_rx) = mpsc::channel::<LifecycleJob>();
        let (result_tx, result_rx) = mpsc::channel::<LifecycleResult>();
        let join = thread::Builder::new()
            .name("residiuum-seal-pipeline".into())
            .spawn(move || worker_loop(job_rx, result_tx))
            .expect("spawn seal pipeline worker");
        Self {
            job_tx,
            result_rx,
            join: Some(join),
            inflight_seals: 0,
            max_pending_seals: DEFAULT_MAX_PENDING_SEALS,
        }
    }

    /// Submit a seal finalize job. Caller tracks `inflight_seals`.
    pub fn submit_seal(&self, job: LifecycleJob) -> Result<(), StoreError> {
        self.job_tx.send(job).map_err(|_| {
            StoreError::Io(std::io::Error::new(
                std::io::ErrorKind::BrokenPipe,
                "seal pipeline worker gone",
            ))
        })
    }

    /// Submit a checkpoint job (does not count against seal inflight).
    pub fn submit_checkpoint(&self, job: LifecycleJob) -> Result<(), StoreError> {
        self.submit_seal(job)
    }

    /// Non-blocking poll for one completed result.
    pub fn try_recv(&self) -> Option<LifecycleResult> {
        match self.result_rx.try_recv() {
            Ok(r) => Some(r),
            Err(TryRecvError::Empty) | Err(TryRecvError::Disconnected) => None,
        }
    }

    /// Block until one result arrives (or worker disconnects).
    pub fn recv(&self) -> Option<LifecycleResult> {
        self.result_rx.recv().ok()
    }

    /// Block with timeout for one result.
    pub fn recv_timeout(&self, timeout: Duration) -> Option<LifecycleResult> {
        match self.result_rx.recv_timeout(timeout) {
            Ok(r) => Some(r),
            Err(RecvTimeoutError::Timeout) | Err(RecvTimeoutError::Disconnected) => None,
        }
    }

    /// Shut down the worker and join. Best-effort; pending jobs may complete first.
    pub fn shutdown(mut self) {
        let _ = self.job_tx.send(LifecycleJob::Shutdown);
        if let Some(h) = self.join.take() {
            let _ = h.join();
        }
        // Drain residual results so the channel is empty.
        while self.result_rx.try_recv().is_ok() {}
    }
}

impl Drop for SealPipeline {
    fn drop(&mut self) {
        let _ = self.job_tx.send(LifecycleJob::Shutdown);
        if let Some(h) = self.join.take() {
            let _ = h.join();
        }
    }
}

fn worker_loop(job_rx: Receiver<LifecycleJob>, result_tx: Sender<LifecycleResult>) {
    while let Ok(job) = job_rx.recv() {
        match job {
            LifecycleJob::Shutdown => break,
            LifecycleJob::FinalizeSeal {
                store_id,
                segment_id,
                pending_path,
                sealed_path,
                limits,
                paths,
                require_fsync,
            } => {
                match finalize_seal(
                    store_id,
                    segment_id,
                    &pending_path,
                    &sealed_path,
                    limits,
                    &paths,
                    require_fsync,
                ) {
                    Ok((content_hash, size, sealed_bytes)) => {
                        let _ = result_tx.send(LifecycleResult::SealDone {
                            segment_id,
                            content_hash,
                            size,
                            sealed_bytes,
                        });
                    }
                    Err(e) => {
                        let _ = result_tx.send(LifecycleResult::SealFailed {
                            segment_id,
                            error: e.to_string(),
                        });
                    }
                }
            }
            LifecycleJob::Checkpoint {
                cache_path,
                store_id,
                frontier,
                index,
            } => {
                let ok = write_primary_index_frontier(&cache_path, store_id, &frontier, &index).is_ok();
                let _ = result_tx.send(LifecycleResult::CheckpointDone { ok });
            }
        }
    }
}

/// Finalize one pending segment: seal (preserve offsets), publish sealed image,
/// BLAKE3, Hydra, best-effort Chimera from segment puts, remove pending.
///
/// **Hot path:** append only the segment-summary suffix to the pending file and
/// `rename` into `segments/` (no full ~seal-threshold rewrite). Falls back to
/// write-temp+rename if rename-across-volume fails.
pub fn finalize_seal(
    store_id: [u8; 16],
    segment_id: [u8; 16],
    pending_path: &std::path::Path,
    sealed_path: &std::path::Path,
    limits: SafetyLimits,
    paths: &StorePaths,
    require_fsync: bool,
) -> Result<([u8; 32], u64, Vec<u8>), StoreError> {
    if !pending_path.is_file() {
        // Already finalized (retry / recover race).
        if sealed_path.is_file() {
            let bytes = fs::read(sealed_path)?;
            let hash = *blake3::hash(&bytes).as_bytes();
            return Ok((hash, bytes.len() as u64, bytes));
        }
        return Err(StoreError::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("pending seal missing: {}", pending_path.display()),
        )));
    }

    let raw = fs::read(pending_path)?;
    let (sealed_bytes, prefix_len) = seal_pending_bytes(raw, store_id, segment_id, limits)?;
    let content_hash = *blake3::hash(&sealed_bytes).as_bytes();
    let size = sealed_bytes.len() as u64;
    debug_assert!(prefix_len as usize <= sealed_bytes.len());
    debug_assert!(
        sealed_bytes.len() >= prefix_len as usize,
        "summary must extend the verified prefix"
    );

    if let Some(parent) = sealed_path.parent() {
        fs::create_dir_all(parent)?;
    }

    // Prefer: truncate pending to verified prefix, append summary only, rename
    // into segments/. Avoids rewriting tens of MiB already on disk.
    let published = publish_sealed_from_pending(
        pending_path,
        sealed_path,
        &sealed_bytes,
        prefix_len,
        require_fsync,
    )?;
    if !published {
        // Cross-device or exotic FS: full write to temp + rename (old path).
        let tmp = sealed_path.with_extension("residiuum.tmp");
        {
            let mut out = OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(&tmp)?;
            out.write_all(&sealed_bytes)?;
            if require_fsync {
                out.sync_all()?;
            }
        }
        fs::rename(&tmp, sealed_path)?;
        if require_fsync {
            if let Some(parent) = sealed_path.parent() {
                let _ = crate::atomic_file::sync_dir(parent);
            }
        }
        let _ = fs::remove_file(pending_path);
        if require_fsync {
            if let Some(parent) = pending_path.parent() {
                let _ = crate::atomic_file::sync_dir(parent);
            }
        }
    }

    // Derived indexes (non-fatal).
    let _ = write_hydra_for_bytes(paths, store_id, segment_id, &sealed_bytes, limits);
    let _ = write_chimera_from_segment_puts(paths, store_id, segment_id, &sealed_bytes, limits);

    Ok((content_hash, size, sealed_bytes))
}

/// Append summary to pending and rename to sealed. Returns false if rename failed
/// (caller should fall back to full write).
fn publish_sealed_from_pending(
    pending_path: &std::path::Path,
    sealed_path: &std::path::Path,
    sealed_bytes: &[u8],
    prefix_len: u64,
    require_fsync: bool,
) -> Result<bool, StoreError> {
    let prefix = prefix_len as usize;
    if prefix > sealed_bytes.len() {
        return Ok(false);
    }
    let summary = &sealed_bytes[prefix..];
    // In-place: keep verified prefix, append summary frame(s) only.
    {
        // Truncate first (separate handle), then append summary — avoids
        // platform-dependent seek-after-set_len positioning bugs.
        {
            let f = OpenOptions::new().write(true).open(pending_path)?;
            f.set_len(prefix_len)?;
        }
        let mut f = OpenOptions::new().append(true).open(pending_path)?;
        f.write_all(summary)?;
        if require_fsync {
            f.sync_all()?;
        }
    }
    // Destination must not exist for rename (Windows / some FS).
    if sealed_path.exists() {
        let _ = fs::remove_file(sealed_path);
    }
    match fs::rename(pending_path, sealed_path) {
        Ok(()) => {
            if require_fsync {
                if let Some(parent) = sealed_path.parent() {
                    let _ = crate::atomic_file::sync_dir(parent);
                }
            }
            Ok(true)
        }
        Err(_) => {
            // Leave pending with summary appended; fallback will write sealed_bytes
            // and remove pending.
            Ok(false)
        }
    }
}

/// Synchronously finalize every pending file under `active/pending/` (open recovery).
pub fn recover_all_pending(paths: &StorePaths, store_id: [u8; 16], limits: SafetyLimits) -> Result<usize, StoreError> {
    let dir = paths.pending_seal_dir();
    if !dir.is_dir() {
        return Ok(0);
    }
    let files = list_residiuum_files(&dir)?;
    let mut n = 0;
    for pending_path in files {
        let Some(segment_id) = segment_id_from_filename(&pending_path) else {
            continue;
        };
        let sealed_path = paths.sealed_segment(&segment_id);
        finalize_seal(
            store_id,
            segment_id,
            &pending_path,
            &sealed_path,
            limits,
            paths,
            true, // open recovery: prefer stable sealed publish
        )?;
        n += 1;
    }
    Ok(n)
}

/// List pending seal segment paths (for pread / all_segment_paths).
pub fn list_pending_paths(paths: &StorePaths) -> Result<Vec<PathBuf>, StoreError> {
    list_residiuum_files(&paths.pending_seal_dir()).map_err(StoreError::from)
}

/// Public entry for sealing an unsealed segment image (active or pending file
/// contents) into a sealed image + verified-prefix length.
///
/// Used by async finalize and by sync seal after write-through discard.
pub fn seal_pending_image(
    raw: Vec<u8>,
    store_id: [u8; 16],
    segment_id: [u8; 16],
    limits: SafetyLimits,
) -> Result<(Vec<u8>, u64), StoreError> {
    seal_pending_bytes(raw, store_id, segment_id, limits)
}

/// Returns `(sealed_image, verified_prefix_len)`.
///
/// `verified_prefix_len` is the byte length of the contiguous verified prefix
/// **before** the summary frame is appended — used to append-only publish.
///
/// Hot path avoids a second full-prefix clone: scan, truncate `raw` in place,
/// resume, seal (append summary into the same `Vec`), move out.
fn seal_pending_bytes(
    mut raw: Vec<u8>,
    store_id: [u8; 16],
    segment_id: [u8; 16],
    limits: SafetyLimits,
) -> Result<(Vec<u8>, u64), StoreError> {
    // Contiguous verified prefix only (same discipline as active recovery).
    let report = scan_forward(&raw, limits);
    let mut end = 0u64;
    let mut frame_count = 0u64;
    let mut writer_sequence = 0u64;
    let mut created_ns = 0u64;
    let mut found_id = None;
    for region in &report.regions {
        match region {
            residiuum_format::ScanRegion::VerifiedFrame { range, frame } => {
                if range.start != end {
                    break;
                }
                end = range.end;
                frame_count = frame_count.saturating_add(1);
                writer_sequence = frame.header.writer_sequence.saturating_add(1);
                if frame.header.known_kind() == Some(FrameKind::SegmentDescriptor) {
                    if let Some((ids, ns, _)) = decode_descriptor_body(&frame.body) {
                        if ids.store_id == store_id {
                            found_id = Some(ids.segment_id);
                            created_ns = ns;
                        }
                    }
                }
                // Already sealed? Accept as-is (prefix == full sealed image).
                if frame.header.known_kind() == Some(FrameKind::SegmentSummary) {
                    raw.truncate(end as usize);
                    return Ok((raw, end));
                }
            }
            residiuum_format::ScanRegion::Hole { .. } => break,
        }
    }
    let sid = found_id.unwrap_or(segment_id);
    let prefix_len = end;
    if end == 0 || frame_count == 0 {
        return Err(StoreError::CorruptMeta("pending segment empty or unreadable"));
    }
    // Keep capacity; drop any torn tail past the verified prefix.
    raw.truncate(end as usize);
    // Prefix bytes for integrity check after seal (summary must not rewrite them).
    // We only need to verify length + that seal only appends — compare via
    // prefix_len and that sealed starts with the same length of content by
    // checking sealed_len == prefix + summary and resume used the same Vec.
    let ids = SegmentId::new(store_id, sid);
    let active = ActiveSegment::resume_unsealed(
        ids,
        limits,
        raw,
        frame_count,
        writer_sequence,
        created_ns,
    )
    .map_err(|e| StoreError::CorruptMeta(match e {
        residiuum_format::SegmentError::MissingDescriptor => "pending missing descriptor",
        residiuum_format::SegmentError::AlreadySealed => "pending already sealed",
        _ => "pending resume failed",
    }))?;
    let sealed = active
        .seal()
        .map_err(|_| StoreError::CorruptMeta("pending seal summary failed"))?;
    let sealed_bytes = sealed.into_bytes();
    // Integrity: seal must only append (summary) past the verified prefix.
    if sealed_bytes.len() < prefix_len as usize {
        return Err(StoreError::CorruptMeta(
            "seal summary path failed to preserve verified prefix",
        ));
    }
    Ok((sealed_bytes, prefix_len))
}

fn write_hydra_for_bytes(
    paths: &StorePaths,
    store_id: [u8; 16],
    segment_id: [u8; 16],
    bytes: &[u8],
    limits: SafetyLimits,
) -> Result<(), StoreError> {
    let records = records_from_segment_bytes(bytes, limits);
    if records.is_empty() {
        return Ok(());
    }
    let index = crate::hydra::build(&records, &HydraBuildOptions::default());
    let path = hydra_index_path(paths, &segment_id);
    write_hydra_index(&path, store_id, segment_id, &index)
}

/// Chimera layout from put events in the sealed segment (derived; may include
/// superseded keys that were later deleted on a newer segment — same class of
/// derived approximation as a full live projection when index is unavailable).
fn write_chimera_from_segment_puts(
    paths: &StorePaths,
    store_id: [u8; 16],
    segment_id: [u8; 16],
    bytes: &[u8],
    limits: SafetyLimits,
) -> Result<(), StoreError> {
    use crate::envelope::decode_item_envelope;
    use residiuum_format::scan_forward;

    let report = scan_forward(bytes, limits);
    let mut pairs: Vec<(Vec<u8>, Vec<u8>)> = Vec::new();
    // Latest put body per subject within this segment only.
    let mut last: std::collections::BTreeMap<Vec<u8>, Vec<u8>> = std::collections::BTreeMap::new();
    for (_off, frame) in report.verified_frames() {
        if frame.header.known_kind() != Some(FrameKind::ItemEvent) {
            continue;
        }
        let Some(env) = decode_item_envelope(&frame.envelope) else {
            continue;
        };
        match env.event_kind {
            crate::envelope::EventKind::Put => {
                last.insert(env.subject, frame.body.clone());
            }
            crate::envelope::EventKind::Delete => {
                last.remove(&env.subject);
            }
        }
    }
    for (k, v) in last {
        pairs.push((k, v));
    }
    if pairs.is_empty() {
        return Ok(());
    }
    let layout = crate::chimera::build_layout(
        &pairs,
        1,
        &crate::chimera::ClassifyOptions::default(),
    );
    let path = crate::chimera::chimera_layout_path(paths, &segment_id);
    crate::chimera::write_chimera_layout(&path, store_id, segment_id, &layout)
}

#[cfg(test)]
mod tests {
    use super::*;
    use residiuum_format::{ActiveSegment, FrameKind, SegmentId};
    use tempfile::tempdir;

    #[test]
    fn seal_pending_preserves_prefix_bytes() {
        let ids = SegmentId::new([1u8; 16], [2u8; 16]);
        let mut active =
            ActiveSegment::create(ids, SafetyLimits::default(), 42).unwrap();
        let off = active
            .append(FrameKind::ItemEvent, &[0xa0], b"hello", [9u8; 16])
            .unwrap();
        assert!(off > 0);
        let raw = active.as_bytes().to_vec();
        let prefix_len = raw.len();
        let (sealed, kept) =
            seal_pending_bytes(raw.clone(), [1u8; 16], [2u8; 16], SafetyLimits::default())
                .unwrap();
        assert_eq!(kept as usize, prefix_len);
        assert!(sealed.len() > prefix_len);
        assert_eq!(&sealed[..prefix_len], &raw[..]);
        // Original item body still at same offset.
        let (_h, _e, body, _, _) =
            residiuum_format::verify_frame_at(&sealed[off as usize..], SafetyLimits::default())
                .unwrap();
        assert_eq!(body, b"hello");
    }

    #[test]
    fn finalize_append_only_preserves_pending_prefix() {
        let dir = tempdir().unwrap();
        let paths = StorePaths::new(dir.path());
        paths.create_dirs().unwrap();
        let store_id = [1u8; 16];
        let segment_id = [2u8; 16];
        let ids = SegmentId::new(store_id, segment_id);
        let mut active = ActiveSegment::create(ids, SafetyLimits::default(), 1).unwrap();
        let off = active
            .append(FrameKind::ItemEvent, &[0xa0], b"hello", [9u8; 16])
            .unwrap();
        let pending = paths.pending_segment(&segment_id);
        let raw = active.as_bytes().to_vec();
        let prefix_len = raw.len();
        fs::write(&pending, &raw).unwrap();
        let sealed = paths.sealed_segment(&segment_id);
        let (_hash, size, bytes) = finalize_seal(
            store_id,
            segment_id,
            &pending,
            &sealed,
            SafetyLimits::default(),
            &paths,
            false,
        )
        .unwrap();
        assert!(sealed.is_file());
        assert!(!pending.is_file());
        assert_eq!(size, bytes.len() as u64);
        assert_eq!(&bytes[..prefix_len], &raw[..]);
        let on_disk = fs::read(&sealed).unwrap();
        assert_eq!(on_disk, bytes);
        let (_h, _e, body, _, _) =
            residiuum_format::verify_frame_at(&on_disk[off as usize..], SafetyLimits::default())
                .unwrap();
        assert_eq!(body, b"hello");
    }

    #[test]
    fn finalize_roundtrip_files() {
        let dir = tempdir().unwrap();
        let paths = StorePaths::new(dir.path());
        paths.create_dirs().unwrap();
        let store_id = [7u8; 16];
        let segment_id = [8u8; 16];
        let ids = SegmentId::new(store_id, segment_id);
        let mut active = ActiveSegment::create(ids, SafetyLimits::default(), 1).unwrap();
        active
            .append(FrameKind::ItemEvent, &[0xa0], b"body", [3u8; 16])
            .unwrap();
        let pending = paths.pending_segment(&segment_id);
        fs::write(&pending, active.as_bytes()).unwrap();
        let sealed = paths.sealed_segment(&segment_id);
        let (hash, size, bytes) = finalize_seal(
            store_id,
            segment_id,
            &pending,
            &sealed,
            SafetyLimits::default(),
            &paths,
            true,
        )
        .unwrap();
        assert!(sealed.is_file());
        assert!(!pending.is_file());
        assert_eq!(size, bytes.len() as u64);
        assert_eq!(hash, *blake3::hash(&bytes).as_bytes());
    }
}