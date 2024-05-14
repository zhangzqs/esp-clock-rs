mod app_name;
mod message;
mod topic;

pub use {
    app_name::AppName,
    message::{Message, SchedulerMessage},
    topic::Topic,
};

#[derive(Debug, Clone, Copy)]
pub enum MessageTo {
    Broadcast,
    App(AppName),
    Topic(Topic),
}

pub trait Context {
    // 发送消息
    fn send_message(&self, to: MessageTo, msg: Message);

    // 订阅话题消息
    fn subscribe_topic_message(&self, topic: Topic);

    // 退出app
    fn exit(self);
}

pub trait App {
    // app名称
    fn app_name(&self) -> AppName;

    // 当app收到消息时
    fn handle_message(&self, ctx: Box<dyn Context>, from: AppName, to: MessageTo, msg: Message);
}
