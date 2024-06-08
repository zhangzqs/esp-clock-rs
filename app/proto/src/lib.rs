pub mod ipc;
pub mod storage;

pub mod message;
mod node;
mod topic;

use std::rc::Rc;

use serde::{Deserialize, Serialize};

pub use {message::*, node::NodeName, topic::TopicName};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MessageTo {
    Broadcast,
    Point(NodeName),
    Topic(TopicName),
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

pub trait WaitGroup {
    fn inc(&self);
    fn done(&self);
    fn wait(&self, callback: Box<dyn FnOnce()>);
}

pub trait Context {
    // 发送广播消息
    fn broadcast_global(&self, msg: Message);

    // 发送话题消息
    fn broadcast_topic(&self, topic: TopicName, msg: Message);

    // 订阅话题
    fn subscribe_topic(&self, topic: TopicName);

    // 解除订阅话题
    fn unsubscribe_topic(&self, topic: TopicName);

    // 发送只会反馈一次的消息
    fn async_call(&self, node: NodeName, msg: Message, callback: MessageCallbackOnce);

    // 发送同步消息
    fn sync_call(&self, node: NodeName, msg: Message) -> HandleResult;

    // 消息就绪，并传递值
    fn async_ready(&self, seq: usize, result: Message);

    // 创建等待器
    fn create_wait_group(&self) -> Rc<dyn WaitGroup>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HandleResult {
    // 成功处理消息，发送方收到一个反馈响应回调消息
    Finish(Message),
    // 消息被丢弃，发送方也得不到响应回调(仅调度器可感知该消息结果)
    Discard,
    // 消息还在处理，下一轮将继续被轮询(仅调度器可感知该消息结果)
    Pending,
    // 对于广播消息，当某个节点返回该结果时，将阻断继续广播
    Block,
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
