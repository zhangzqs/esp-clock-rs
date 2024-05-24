use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LifecycleMessage {
    // 调度器首次调度向所有组件发送一个初始化消息
    Init,
    // 当组件可见时
    Show,
    // 当组件不可见时
    Hide,
}
