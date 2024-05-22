pub mod ipc;
mod message;
mod node;
use std::{rc::Rc, time::Duration};

pub use {message::*, node::NodeName};

#[derive(Debug, Clone, Copy)]
pub enum MessageTo {
    Broadcast,
    Point(NodeName),
}

pub type MessageCallbackOnce = Box<dyn FnOnce(NodeName, HandleResult)>;

pub trait Context {
    // 发送一条消息，无反馈
    fn send_message(&self, to: MessageTo, msg: Message);

    fn send_message_with_timeout_and_reply_once(
        &self,
        to: MessageTo,
        msg: Message,
        timeout: Option<Duration>,
        callback: MessageCallbackOnce,
    );

    // 发送只会反馈一次的消息
    fn send_message_with_reply_once(
        &self,
        to: MessageTo,
        msg: Message,
        callback: MessageCallbackOnce,
    ) {
        self.send_message_with_timeout_and_reply_once(to, msg, None, callback);
    }
}

#[derive(Debug, Clone)]
pub enum HandleResult {
    // 成功处理消息，发送方收到一个反馈响应回调消息
    Successful(Message),
    // 消息被丢弃，发送方也得不到响应回调(仅调度器可感知该消息结果)
    Discard,
    // 消息处理失败，发送方收到一个响应回调错误消息
    Error(Message),
    // 消息还在处理，下一轮将继续被轮询(仅调度器可感知该消息结果)
    Pending,
    // 消息处理超时
    Timeout,
}

impl HandleResult {
    pub fn map<T, E>(
        self,
        value_mapper: impl FnOnce(Message) -> T,
        error_mapper: impl FnOnce(Message) -> E,
    ) -> Result<T, E> {
        match self {
            HandleResult::Successful(e) => Ok(value_mapper(e)),
            HandleResult::Error(e) => Err(error_mapper(e)),
            _ => {
                panic!("cannot map HandleResult {:?} into Result", self)
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct MessageWithHeader {
    /// 消息帧ID
    pub seq: u32,
    /// 消息超时时间点，相对于调度器首次调度的时间点
    pub timeout: Option<Duration>,
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
