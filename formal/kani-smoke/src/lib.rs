//! FAS-1 Kani smoke harness.
#[cfg(kani)]
#[kani::proof]
fn fas1_kani_smoke() {
    let x: u8 = kani::any();
    kani::assume(x < 10);
    assert!(x < 10);
}
