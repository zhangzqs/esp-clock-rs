use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    rc::Rc,
};

use crate::common::*;
struct ContextImpl {
    app_name: AppName,
    mq_buffer: Rc<RefCell<Vec<(AppName, MessageTo, Message)>>>,
    topic_subscriber: Rc<RefCell<HashMap<Topic, HashSet<AppName>>>>,
}

impl Context for ContextImpl {
    // 发送消息
    fn send_message(&self, to: MessageTo, msg: Message) {
        self.mq_buffer.borrow_mut().push((self.app_name, to, msg))
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
}

pub struct Scheduler {
    apps: HashMap<AppName, Box<dyn App>>,
    mq_buffer1: RefCell<Vec<(AppName, MessageTo, Message)>>,
    mq_buffer2: Rc<RefCell<Vec<(AppName, MessageTo, Message)>>>,
    topic_subscriber: Rc<RefCell<HashMap<Topic, HashSet<AppName>>>>,
}

impl Scheduler {
    pub fn new() -> Self {
        Self {
            apps: HashMap::new(),
            mq_buffer1: RefCell::new(vec![(
                AppName::Scheduler,
                MessageTo::Broadcast,
                Message::Scheduler(SchedulerMessage::Start), // 首次启动先广播一个开始调度消息
            )]),
            mq_buffer2: Rc::new(RefCell::new(Vec::new())),
            topic_subscriber: Rc::new(RefCell::new(HashMap::new())),
        }
    }

    pub fn register_app<A: App + 'static>(&mut self, app: A) {
        self.apps.insert(app.app_name(), Box::new(app));
    }

    pub fn schedule_once(&mut self) {
        // 消费消息
        for (from, to, msg) in self.mq_buffer1.borrow_mut().drain(..) {
            match to {
                MessageTo::Broadcast => {
                    for (app_name, app) in self.apps.iter_mut() {
                        app.handle_message(
                            Box::new(ContextImpl {
                                app_name: *app_name,
                                mq_buffer: self.mq_buffer2.clone(),
                                topic_subscriber: self.topic_subscriber.clone(),
                            }),
                            from,
                            to,
                            msg.clone(),
                        )
                    }
                }
                MessageTo::App(app_name) => {
                    self.apps.entry(app_name).and_modify(|x| {
                        x.handle_message(
                            Box::new(ContextImpl {
                                app_name: app_name,
                                mq_buffer: self.mq_buffer2.clone(),
                                topic_subscriber: self.topic_subscriber.clone(),
                            }),
                            from,
                            to,
                            msg.clone(),
                        );
                    });
                }
                MessageTo::Topic(topic) => {
                    if let Some(apps) = self.topic_subscriber.borrow().get(&topic) {
                        for app_name in apps.iter() {
                            self.apps.entry(*app_name).and_modify(|x| {
                                x.handle_message(
                                    Box::new(ContextImpl {
                                        app_name: *app_name,
                                        mq_buffer: self.mq_buffer2.clone(),
                                        topic_subscriber: self.topic_subscriber.clone(),
                                    }),
                                    from,
                                    to,
                                    msg.clone(),
                                )
                            });
                        }
                    }
                }
            }
        }

        // 交换两个缓冲区队列
        self.mq_buffer2.swap(&self.mq_buffer1)
    }
}
