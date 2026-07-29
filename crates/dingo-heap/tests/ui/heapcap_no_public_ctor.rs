fn main() {
    let _cap = dingo_heap::HeapCap {
        inner: std::sync::Arc::new(loop {}),
    };
}
