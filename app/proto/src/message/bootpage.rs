use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BootPageMessage {
    EnablePerformanceMonitor(bool),
}
