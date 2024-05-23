#[derive(Debug, Clone)]
pub enum PerformanceMessage {
    GetFreeHeapSizeRequest,
    GetFreeHeapSizeResponse(usize),
    GetLargestFreeBlock,
    GetLargestFreeBlockResponse(usize),
    GetFpsRequest,
    GetFpsResponse(usize),
}
