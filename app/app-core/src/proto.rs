pub mod ipc;
mod message;
mod node;
use std::{rc::Rc};

pub use {message::*, node::NodeName};

#[derive(Debug, Clone, Copy)]
pub enum MessageTo {
    Broadcast,
    Point(NodeName),
}

pub type MessageCallbackOnce = Box<dyn FnOnce(HandleResult)>;

pub trait Context {
    // 发送一条消息，无反馈
    fn send_message(&self, to: MessageTo, msg: Message);

    // 发送只会反馈一次的消息
    fn send_message_with_reply_once(
        &self,
        to: MessageTo,
        msg: Message,
        callback: MessageCallbackOnce,
    );
}

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub struct MessageWithHeader {
    /// 消息帧ID
    pub seq: u32,
    /// 消息是否处于pending态
    pub is_pending: bool,
    /// 消息体
    pub body: Message,
}

pub trait Node {
    // 节点名称
    fn node_name(&self) -> NodeName;

    // 当节点收到消息时
    fn handle_message(
        &mut self,
        _ctx: Rc<dyn Context>,
        _from: NodeName,
        _to: MessageTo,
        _msg: MessageWithHeader,
    ) -> HandleResult {
        HandleResult::Discard
    }
}
