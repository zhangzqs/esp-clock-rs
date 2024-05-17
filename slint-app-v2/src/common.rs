mod app_name;
mod message;
mod topic;

pub use {app_name::AppName, message::*, topic::Topic};

#[derive(Debug, Clone, Copy)]
pub enum MessageTo {
    Broadcast,
    App(AppName),
    Topic(Topic),
}

pub type MessageCallbackOnce = Box<dyn FnOnce(AppName, HandleResult)>;
pub type MessageCallback = Box<dyn Fn(AppName, HandleResult)>;

pub trait Context {
    // 发送消息无反馈
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

    // 订阅话题消息
    fn subscribe_topic_message(&self, topic: Topic);

    // 取消订阅话题消息
    fn unsubscribe_topic_message(&self, topic: Topic);
}

#[derive(Debug, Clone)]
pub enum HandleResult {
    // 成功处理消息，发送方收到一个反馈响应回调消息
    Successful(Message),
    // 消息被丢弃，发送方也得不到响应回调
    Discard,
    // 消息处理失败，发送方收到一个响应回调错误消息
    Error(Message),
}

pub trait App {
    // app名称
    fn app_name(&self) -> AppName;

    // 当app收到消息时
    fn handle_message(
        &mut self,
        _ctx: Box<dyn Context>,
        _from: AppName,
        _to: MessageTo,
        _msg: Message,
    ) -> HandleResult;
}