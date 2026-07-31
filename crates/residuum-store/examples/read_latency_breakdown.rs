//! Read-path phase breakdown (response to ~250 ms testrig get samples).
//!
//! Proves that a healthy open store does **one locator lookup + resident body
//! return** (or one bounded disk read when probing Chimera), with no open,
//! salvage, catalog rebuild, or full-segment sidecar reload on the hot path.
//!
//! Surfaces measured (one process, already-open DB unless noted):
//! - `open_db` / `open_inspect`
//! - same key × N (resident PrimaryIndex)
//! - N distinct keys (resident PrimaryIndex)
//! - reopen-per-get (anti-pattern; expected slow)
//! - open-once then N gets
//! - optional Chimera probe (full `.cmr` load; diagnostic only)
//!
//! Run: `cargo run -p residuum-store --release --example read_latency_breakdown`

use residuum_store::{DurabilityMode, Store};
use std::path::Path;
use std::time::{Duration, Instant};

fn mean_us(samples: &[Duration]) -> f64 {
    if samples.is_empty() {
        return 0.0;
    }
    samples.iter().map(|d| d.as_secs_f64() * 1e6).sum::<f64>() / samples.len() as f64
}

fn pctl_us(samples: &mut [Duration], p: f64) -> f64 {
    if samples.is_empty() {
        return 0.0;
    }
    samples.sort();
    let idx = ((samples.len() as f64 - 1.0) * p).round() as usize;
    samples[idx.min(samples.len() - 1)].as_secs_f64() * 1e6
}

fn summarize(label: &str, mut samples: Vec<Duration>) {
    let n = samples.len();
    let mean = mean_us(&samples);
    let p50 = pctl_us(&mut samples, 0.50);
    let p95 = pctl_us(&mut samples, 0.95);
    let p99 = pctl_us(&mut samples, 0.99);
    let max = samples.last().map(|d| d.as_secs_f64() * 1e6).unwrap_or(0.0);
    eprintln!(
        "{label:>32}: n={n} mean={mean:10.1}µs p50={p50:10.1}µs p95={p95:10.1}µs \
         p99={p99:10.1}µs max={max:10.1}µs"
    );
    println!(
        "{{\"phase\":\"{label}\",\"n\":{n},\"mean_us\":{mean:.3},\"p50_us\":{p50:.3},\
         \"p95_us\":{p95:.3},\"p99_us\":{p99:.3},\"max_us\":{max:.3}}}"
    );
}

fn seed_store(path: &Path, n_keys: usize, payload: &[u8], seal_every: usize) {
    let mut store = Store::create(path).expect("create");
    store.set_seal_threshold(4 * 1024 * 1024);
    for i in 0..n_keys {
        let key = format!("k{i:08}");
        store
            .put(&key, payload, DurabilityMode::Buffered)
            .expect("put");
        if seal_every > 0 && (i + 1) % seal_every == 0 {
            let _ = store.seal_active();
        }
    }
    let _ = store.seal_active();
    let _ = store.persist_index_cache();
}

fn main() {
    let dir = tempfile::tempdir().expect("tmp");
    let path = dir.path().join("read_breakdown");
    let n_keys = 4_000usize;
    let payload = vec![0xABu8; 4 * 1024];
    // Seal often enough that Chimera sidecars exist for many segments.
    seed_store(&path, n_keys, &payload, 512);

    // --- open costs ---
    let t0 = Instant::now();
    let store = Store::open(&path).expect("open");
    let open_us = t0.elapsed().as_secs_f64() * 1e6;
    eprintln!("{:>32}: {open_us:.1}µs", "open_db");
    println!("{{\"phase\":\"open_db\",\"elapsed_us\":{open_us:.3}}}");

    let t0 = Instant::now();
    let inspect = Store::open_inspect(&path).expect("open_inspect");
    let inspect_us = t0.elapsed().as_secs_f64() * 1e6;
    eprintln!("{:>32}: {inspect_us:.1}µs", "open_inspect");
    println!("{{\"phase\":\"open_inspect\",\"elapsed_us\":{inspect_us:.3}}}");

    // Warm one get so later windows are steady-state.
    let _ = store.get("k00000000").expect("warm");

    // --- same key × N (resident PrimaryIndex) ---
    let n = 1_000usize;
    let mut same = Vec::with_capacity(n);
    for _ in 0..n {
        let t0 = Instant::now();
        let v = store.get("k00000000").expect("get");
        same.push(t0.elapsed());
        assert_eq!(v.as_deref().map(|b| b.len()), Some(payload.len()));
    }
    summarize("same_key_primary_index", same);

    // --- N distinct keys ---
    let mut distinct = Vec::with_capacity(n);
    for i in 0..n {
        let key = format!("k{i:08}");
        let t0 = Instant::now();
        let v = store.get(&key).expect("get");
        distinct.push(t0.elapsed());
        assert!(v.is_some());
    }
    summarize("random_keys_primary_index", distinct);

    // --- open-once inspect gets (matches testrig monitor pattern) ---
    let mut inspect_gets = Vec::with_capacity(n);
    for i in 0..n {
        let key = format!("k{i:08}");
        let t0 = Instant::now();
        let v = inspect.get(&key).expect("inspect get");
        inspect_gets.push(t0.elapsed());
        assert!(v.is_some());
    }
    summarize("inspect_open_once_gets", inspect_gets);

    // --- anti-pattern: reopen per get ---
    let reopen_n = 32usize;
    let mut reopen = Vec::with_capacity(reopen_n);
    drop(store);
    for i in 0..reopen_n {
        let key = format!("k{i:08}");
        let t0 = Instant::now();
        let s = Store::open_inspect(&path).expect("reopen");
        let v = s.get(&key).expect("get");
        reopen.push(t0.elapsed());
        assert!(v.is_some());
    }
    summarize("reopen_inspect_per_get", reopen);

    // --- Chimera probe (full sidecar load; not the product hot path) ---
    let store = Store::open(&path).expect("reopen writer");
    let mut chimera = Vec::with_capacity(64);
    for i in 0..64 {
        let key = format!("k{i:08}");
        let t0 = Instant::now();
        let _ = store.get_via_chimera(&key).expect("chimera");
        chimera.push(t0.elapsed());
    }
    summarize("chimera_probe_full_sidecar", chimera);

    eprintln!();
    eprintln!("Interpretation:");
    eprintln!(
        "  same_key / random_keys / inspect_open_once → should be µs-class (resident index)."
    );
    eprintln!("  reopen_inspect_per_get → open + rebuild cost per sample (anti-pattern).");
    eprintln!("  chimera_probe_full_sidecar → full .cmr read/decode; diagnostic only.");
    eprintln!("  If every get is ~250ms, the harness is reopening or probing Chimera every time.");
}
