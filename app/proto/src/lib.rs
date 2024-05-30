// pub mod ipc;
mod message;
mod node;
use std::rc::Rc;

use serde::{Deserialize, Serialize};

pub use {message::*, node::NodeName};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageTo {
    Broadcast,
    Point(NodeName),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageWithHeader {
    /// 消息来源
    pub from: NodeName,
    /// 消息发送目标
    pub to: MessageTo,
    /// 消息帧ID
    pub seq: usize,
    /// 消息体
    pub body: Message,
}

pub type MessageCallbackOnce = Box<dyn FnOnce(HandleResult)>;

pub trait Context {
    // 发送广播消息
    fn boardcast(&self, msg: Message);

    // 发送只会反馈一次的消息
    fn async_call(&self, node: NodeName, msg: Message, callback: MessageCallbackOnce);

    // 发送同步消息
    fn sync_call(&self, node: NodeName, msg: Message) -> HandleResult;

    // 消息就绪，并传递值
    fn async_ready(&self, seq: usize, result: Message);
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HandleResult {
    // 成功处理消息，发送方收到一个反馈响应回调消息
    Finish(Message),
    // 消息被丢弃，发送方也得不到响应回调(仅调度器可感知该消息结果)
    Discard,
    // 消息还在处理，下一轮将继续被轮询(仅调度器可感知该消息结果)
    Pending,
}

impl HandleResult {
    pub fn unwrap(self) -> Message {
        match self {
            HandleResult::Finish(m) => m,
            _ => panic!("unexpected HandleResult {:?}", self),
        }
    }
}

pub trait Node {
    // 节点调度优先级，默认为0，越高越优先被调度
    fn priority(&self) -> usize {
        0
    }

    // 节点名称
    fn node_name(&self) -> NodeName;

    // 当节点收到消息时
    fn handle_message(&self, _ctx: Rc<dyn Context>, _msg: MessageWithHeader) -> HandleResult {
        HandleResult::Discard
    }

    // 不断轮询消息是否就绪
    fn poll(&self, _ctx: Rc<dyn Context>, _seq: usize) {}
}
