use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    rc::Rc,
};

use crate::common::*;

struct MessageQueueItem {
    from: AppName,
    to: MessageTo,
    message: Message,
    callback_once: Option<MessageCallbackOnce>,
    callback: Option<MessageCallback>,
}

struct ContextImpl {
    app_name: AppName,
    mq_buffer: Rc<RefCell<Vec<MessageQueueItem>>>,
    topic_subscriber: Rc<RefCell<HashMap<Topic, HashSet<AppName>>>>,
}

impl Context for ContextImpl {
    // 发送消息
    fn send_message(&self, to: MessageTo, msg: Message) {
        self.mq_buffer.borrow_mut().push(MessageQueueItem {
            from: self.app_name,
            to,
            message: msg,
            callback_once: None,
            callback: None,
        })
    }

    // 订阅话题消息
    fn subscribe_topic_message(&self, topic: Topic) {
        self.topic_subscriber
            .borrow_mut()
            .entry(topic)
            .or_default()
            .insert(self.app_name);
    }

    fn unsubscribe_topic_message(&self, topic: Topic) {
        self.topic_subscriber
            .borrow_mut()
            .entry(topic)
            .and_modify(|x| {
                x.remove(&self.app_name);
            });
    }

    fn send_message_with_reply_once(
        &self,
        to: MessageTo,
        msg: Message,
        callback: MessageCallbackOnce,
    ) {
        self.mq_buffer.borrow_mut().push(MessageQueueItem {
            from: self.app_name,
            to,
            message: msg,
            callback_once: Some(callback),
            callback: None,
        })
    }

    fn send_message_with_reply(&self, to: MessageTo, msg: Message, callback: MessageCallback) {
        self.mq_buffer.borrow_mut().push(MessageQueueItem {
            from: self.app_name,
            to,
            message: msg,
            callback_once: None,
            callback: Some(callback),
        })
    }
}

pub struct Scheduler {
    apps: HashMap<AppName, Box<dyn App>>,
    mq_buffer1: RefCell<Vec<MessageQueueItem>>,
    mq_buffer2: Rc<RefCell<Vec<MessageQueueItem>>>,
    topic_subscriber: Rc<RefCell<HashMap<Topic, HashSet<AppName>>>>,
}

impl Default for Scheduler {
    fn default() -> Self {
        Self::new()
    }
}

impl Scheduler {
    pub fn new() -> Self {
        Self {
            apps: HashMap::new(),
            mq_buffer1: RefCell::new(vec![MessageQueueItem {
                from: AppName::Scheduler,
                to: MessageTo::Broadcast,
                message: Message::Lifecycle(LifecycleMessage::Init),
                callback: None,
                callback_once: None,
            }]),
            mq_buffer2: Rc::new(RefCell::new(Vec::new())),
            topic_subscriber: Rc::new(RefCell::new(HashMap::new())),
        }
    }

    pub fn register_app<A: App + 'static>(&mut self, app: A) {
        self.apps.insert(app.app_name(), Box::new(app));
    }

    pub fn schedule_once(&mut self) {
        // 消费消息
        for MessageQueueItem {
            from,
            to,
            message,
            mut callback_once,
            callback,
        } in self.mq_buffer1.borrow_mut().drain(..)
        {
            println!("from: {:?}, to: {:?}, msg: {:?}", from, to, message);
            match to {
                MessageTo::Broadcast => {
                    for (app_name, app) in self.apps.iter_mut() {
                        let ret = app.handle_message(
                            Box::new(ContextImpl {
                                app_name: *app_name,
                                mq_buffer: self.mq_buffer2.clone(),
                                topic_subscriber: self.topic_subscriber.clone(),
                            }),
                            from,
                            to,
                            message.clone(),
                        );
                        if let Some(cb) = callback_once.take() {
                            cb(*app_name, ret.clone());
                        }
                        if let Some(ref cb) = callback {
                            cb(*app_name, ret);
                        }
                    }
                }
                MessageTo::App(app_name) => {
                    self.apps.entry(app_name).and_modify(|x| {
                        let ret = x.handle_message(
                            Box::new(ContextImpl {
                                app_name,
                                mq_buffer: self.mq_buffer2.clone(),
                                topic_subscriber: self.topic_subscriber.clone(),
                            }),
                            from,
                            to,
                            message.clone(),
                        );
                        if let Some(cb) = callback_once {
                            cb(app_name, ret.clone());
                        }
                        if let Some(ref cb) = callback {
                            cb(app_name, ret);
                        }
                    });
                }
                MessageTo::Topic(topic) => {
                    if let Some(apps) = self.topic_subscriber.borrow().get(&topic) {
                        for app_name in apps.iter() {
                            let mut ret = Option::<HandleResult>::None;
                            self.apps.entry(*app_name).and_modify(|x| {
                                let ret1 = x.handle_message(
                                    Box::new(ContextImpl {
                                        app_name: *app_name,
                                        mq_buffer: self.mq_buffer2.clone(),
                                        topic_subscriber: self.topic_subscriber.clone(),
                                    }),
                                    from,
                                    to,
                                    message.clone(),
                                );
                                if let Some(cb) = callback_once.take() {
                                    cb(*app_name, ret1.clone());
                                }
                                ret = Some(ret1);
                            });
                            if let Some(ret) = ret {
                                if let Some(ref cb) = callback {
                                    cb(*app_name, ret);
                                }
                            }
                        }
                    }
                }
            }
        }
        // 交换两个缓冲区队列
        self.mq_buffer2.swap(&self.mq_buffer1)
    }
}
