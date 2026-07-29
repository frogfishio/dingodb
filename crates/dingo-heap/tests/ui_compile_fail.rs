//! trybuild compile-fail suite (HP-001 / HP-003).

#[test]
fn heapcap_not_forgeable() {
    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/heapcap_no_public_ctor.rs");
}
