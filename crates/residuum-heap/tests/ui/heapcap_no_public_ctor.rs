fn main() {
    let _cap = residuum_heap::HeapCap {
        inner: std::sync::Arc::new(loop {}),
    };
}
