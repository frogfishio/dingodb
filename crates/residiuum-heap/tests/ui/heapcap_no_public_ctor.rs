fn main() {
    let _cap = residiuum_heap::HeapCap {
        inner: std::sync::Arc::new(loop {}),
    };
}
