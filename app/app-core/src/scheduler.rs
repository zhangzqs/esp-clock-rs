use std::{
    cell::RefCell,
    collections::HashMap,
    rc::Rc,
    time::{Duration, Instant},
};

use log::debug;

use crate::proto::*;

struct MessageQueueItem {
    from: NodeName,
    to: MessageTo,
    message: MessageWithHeader,
    callback_once: Option<MessageCallbackOnce>,
}

pub trait Platform {
    fn duration_since_init(&self) -> Duration;
}

struct ContextImpl {
    node_name: NodeName,
    mq_buffer: Rc<RefCell<Vec<MessageQueueItem>>>,
    msg_seq_inc: Rc<RefCell<u32>>,
    platform: Rc<dyn Platform>,
}

impl Context for ContextImpl {
    // 发送消息
    fn send_message(&self, to: MessageTo, msg: Message) {
        *self.msg_seq_inc.borrow_mut() += 1;
        self.mq_buffer.borrow_mut().push(MessageQueueItem {
            from: self.node_name,
            to,
            message: MessageWithHeader {
                seq: *self.msg_seq_inc.borrow(),
                body: msg,
                timeout: None,
                is_pending: false,
            },
            callback_once: None,
        })
    }

    fn send_message_with_timeout_and_reply_once(
        &self,
        to: MessageTo,
        msg: Message,
        timeout: Option<Duration>,
        callback: MessageCallbackOnce,
    ) {
        *self.msg_seq_inc.borrow_mut() += 1;
        self.mq_buffer.borrow_mut().push(MessageQueueItem {
            from: self.node_name,
            to,
            message: MessageWithHeader {
                seq: *self.msg_seq_inc.borrow(),
                body: msg,
                timeout: timeout.map(|x| self.platform.duration_since_init() + x),
                is_pending: false,
            },
            callback_once: Some(callback),
        })
    }
}

pub struct Scheduler {
    nodes: HashMap<NodeName, Box<dyn Node>>,
    mq_buffer1: RefCell<Vec<MessageQueueItem>>,
    mq_buffer2: Rc<RefCell<Vec<MessageQueueItem>>>,
    msg_seq_inc: Rc<RefCell<u32>>,
    platform: Rc<dyn Platform>,
}

struct DefaultPlatform {
    start: Instant,
}

impl DefaultPlatform {
    fn new() -> Self {
        Self {
            start: Instant::now(),
        }
    }
}

impl Platform for DefaultPlatform {
    fn duration_since_init(&self) -> Duration {
        self.start.elapsed()
    }
}

impl Scheduler {
    pub fn new() -> Self {
        Self::new_with_platform(DefaultPlatform::new())
    }

    pub fn new_with_platform<P: 'static + Platform>(platform: P) -> Self {
        Self {
            nodes: HashMap::new(),
            mq_buffer1: RefCell::new(vec![MessageQueueItem {
                from: NodeName::Scheduler,
                to: MessageTo::Broadcast,
                message: MessageWithHeader {
                    seq: 0,
                    body: Message::Lifecycle(LifecycleMessage::Init),
                    timeout: None,
                    is_pending: false,
                },
                callback_once: None,
            }]),
            mq_buffer2: Rc::new(RefCell::new(Vec::new())),
            msg_seq_inc: Rc::new(RefCell::new(0)),
            platform: Rc::new(platform),
        }
    }

    pub fn register_node<A: Node + 'static>(&mut self, app: A) {
        self.nodes.insert(app.node_name(), Box::new(app));
    }

    pub fn schedule_once(&mut self) {
        // 心跳消息
        for (node_name, node) in self.nodes.iter_mut() {
            node.handle_message(
                Rc::new(ContextImpl {
                    node_name: *node_name,
                    mq_buffer: self.mq_buffer2.clone(),
                    msg_seq_inc: self.msg_seq_inc.clone(),
                    platform: self.platform.clone(),
                }),
                NodeName::Scheduler,
                MessageTo::Broadcast,
                MessageWithHeader {
                    seq: 0,
                    body: Message::Schedule,
                    timeout: None,
                    is_pending: false,
                },
            );
        }

        // 消费消息
        for MessageQueueItem {
            from,
            to,
            message,
            mut callback_once,
        } in self.mq_buffer1.borrow_mut().drain(..)
        {
            if !message.is_pending {
                debug!(
                    "dispatch message from: {:?}, to: {:?}, msg: {:?}",
                    from, to, message
                );
            }
            match to {
                MessageTo::Broadcast => {
                    for (node_name, node) in self.nodes.iter_mut() {
                        debug!(
                            "handle message from node: {from:?}, to node: {node_name:?}, msg: {}",
                            message.body.debug_msg()
                        );
                        let ret = node.handle_message(
                            Rc::new(ContextImpl {
                                node_name: *node_name,
                                mq_buffer: self.mq_buffer2.clone(),
                                msg_seq_inc: self.msg_seq_inc.clone(),
                                platform: self.platform.clone(),
                            }),
                            from,
                            to,
                            message.clone(),
                        );
                        debug!("handle message result: {ret:?}");
                        match ret {
                            HandleResult::Successful(_) => {
                                if let Some(_) = callback_once.take() {
                                    unimplemented!("broadcast is unsupported for callback")
                                }
                            }
                            HandleResult::Error(_) => {
                                if let Some(_) = callback_once.take() {
                                    unimplemented!("broadcast is unsupported for callback")
                                }
                            }
                            HandleResult::Pending => {
                                unimplemented!("broadcast is unsupported for pending message")
                            }
                            HandleResult::Timeout => {
                                unimplemented!("broadcast is unsupported for timeout")
                            }
                            HandleResult::Discard => {}
                        }
                    }
                }
                MessageTo::Point(node_name) => {
                    self.nodes
                        .entry(node_name)
                        .and_modify(|x| {
                            if !message.is_pending {
                                debug!("handle message from node: {from:?}, to node: {node_name:?}, msg: {}", message.body.debug_msg());
                            }
                            let ret = x.handle_message(
                                Rc::new(ContextImpl {
                                    node_name,
                                    mq_buffer: self.mq_buffer2.clone(),
                                    msg_seq_inc: self.msg_seq_inc.clone(),
                                    platform: self.platform.clone(),
                                }),
                                from,
                                to,
                                message.clone(),
                            );
                            if !message.is_pending {
                                debug!("handle message result: {ret:?}");
                            }

                            match ret {
                                HandleResult::Successful(e) => {
                                    if let Some(cb) = callback_once.take() {
                                        cb(node_name, HandleResult::Successful(e.clone()));
                                    }
                                }
                                HandleResult::Error(e) => {
                                    if let Some(cb) = callback_once.take() {
                                        cb(node_name, HandleResult::Error(e.clone()));
                                    }
                                }
                                HandleResult::Pending => { // 复制一份消息，下一轮pending将继续传递
                                    let mut message = message;
                                    message.is_pending = true;
                                    if let Some(dur) = message.timeout {
                                        if dur < self.platform.duration_since_init() {
                                            // 没有超时
                                            self.mq_buffer2.borrow_mut().push(MessageQueueItem {
                                                from,
                                                to,
                                                message,
                                                callback_once,
                                            });
                                        } else {
                                            // 超时了
                                            if let Some(cb) = callback_once.take() {
                                                cb(node_name, HandleResult::Timeout);
                                            }
                                        }
                                    } else {
                                        // 没有超时
                                        self.mq_buffer2.borrow_mut().push(MessageQueueItem {
                                            from,
                                            to,
                                            message,
                                            callback_once,
                                        });
                                    }
                                }
                                HandleResult::Timeout => {
                                    if let Some(cb) = callback_once.take() {
                                        cb(node_name, HandleResult::Timeout);
                                    }
                                }
                                HandleResult::Discard => {}
                            }
                        })
                        .or_insert_with(|| {
                            // 如果不存在，则panic
                            panic!("not found node {:?}", node_name);
                        });
                }
            }
        }
        // 交换两个缓冲区队列
        self.mq_buffer2.swap(&self.mq_buffer1)
    }
}
