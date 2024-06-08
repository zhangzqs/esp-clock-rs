use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SystemMessage {
    GetFreeHeapSizeRequest,
    GetFreeHeapSizeResponse(usize),
    GetLargestFreeBlock,
    GetLargestFreeBlockResponse(usize),
    GetFpsRequest,
    GetFpsResponse(usize),
    Restart,
}
