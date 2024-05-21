pub mod ipc;
mod message;
mod node;
mod topic;
use std::rc::Rc;
pub use {message::*, node::NodeName, topic::Topic};

#[derive(Debug, Clone, Copy)]
pub enum MessageTo {
    Broadcast,
    Point(NodeName),
    Topic(Topic),
}

pub type MessageCallbackOnce = Box<dyn FnOnce(NodeName, HandleResult)>;
pub type MessageCallback = Box<dyn Fn(NodeName, HandleResult)>;

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

    // 发送可能会反馈多次的消息
    fn send_message_with_reply(&self, to: MessageTo, msg: Message, callback: MessageCallback);

    // 订阅一个话题消息
    fn subscribe_topic_message(&self, topic: Topic);

    // 取消订阅话题消息
    fn unsubscribe_topic_message(&self, topic: Topic);
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
}

#[derive(Debug, Clone)]
pub struct MessageWithHeader {
    /// 消息帧ID
    pub seq: u32,
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
