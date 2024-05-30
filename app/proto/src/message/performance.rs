use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PerformanceMessage {
    GetFreeHeapSizeRequest,
    GetFreeHeapSizeResponse(usize),
    GetLargestFreeBlock,
    GetLargestFreeBlockResponse(usize),
    GetFpsRequest,
    GetFpsResponse(usize),
}
