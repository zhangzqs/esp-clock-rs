use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    rc::Rc,
};

use log::debug;

use proto::*;

struct MessageQueueItem {
    from: NodeName,
    to: MessageTo,
    message: MessageWithHeader,
    callback_once: Option<MessageCallbackOnce>,
    callback: Option<MessageCallback>,
    is_pending: bool,
}

struct ContextImpl {
    node_name: NodeName,
    mq_buffer: Rc<RefCell<Vec<MessageQueueItem>>>,
    topic_subscriber: Rc<RefCell<HashMap<Topic, HashSet<NodeName>>>>,
    msg_seq_inc: Rc<RefCell<u32>>,
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
                body: msg 
            },
            callback_once: None,
            callback: None,
            is_pending: false,
        })
    }

    // 订阅话题消息
    fn subscribe_topic_message(&self, topic: Topic) {
        self.topic_subscriber
            .borrow_mut()
            .entry(topic)
            .or_default()
            .insert(self.node_name);
    }

    fn unsubscribe_topic_message(&self, topic: Topic) {
        self.topic_subscriber
            .borrow_mut()
            .entry(topic)
            .and_modify(|x| {
                x.remove(&self.node_name);
            });
    }

    fn send_message_with_reply_once(
        &self,
        to: MessageTo,
        msg: Message,
        callback: MessageCallbackOnce,
    ) {
        *self.msg_seq_inc.borrow_mut() += 1;
        self.mq_buffer.borrow_mut().push(MessageQueueItem {
            from: self.node_name,
            to,
            message: MessageWithHeader { 
                seq: *self.msg_seq_inc.borrow(),
                body: msg 
            },
            callback_once: Some(callback),
            callback: None,
            is_pending: false,
        })
    }

    fn send_message_with_reply(&self, to: MessageTo, msg: Message, callback: MessageCallback) {
        *self.msg_seq_inc.borrow_mut() += 1;
        self.mq_buffer.borrow_mut().push(MessageQueueItem {
            from: self.node_name,
            to,
            message: MessageWithHeader { 
                seq: *self.msg_seq_inc.borrow(),
                body: msg 
            },
            callback_once: None,
            callback: Some(callback),
            is_pending: false,
        })
    }
}

pub struct Scheduler {
    nodes: HashMap<NodeName, Box<dyn Node>>,
    mq_buffer1: RefCell<Vec<MessageQueueItem>>,
    mq_buffer2: Rc<RefCell<Vec<MessageQueueItem>>>,
    topic_subscriber: Rc<RefCell<HashMap<Topic, HashSet<NodeName>>>>,
    msg_seq_inc: Rc<RefCell<u32>>,
}

impl Default for Scheduler {
    fn default() -> Self {
        Self::new()
    }
}

impl Scheduler {
    pub fn new() -> Self {
        Self {
            nodes: HashMap::new(),
            mq_buffer1: RefCell::new(vec![MessageQueueItem {
                from: NodeName::Scheduler,
                to: MessageTo::Broadcast,
                message: MessageWithHeader { 
                    seq: 0, 
                    body: Message::Lifecycle(LifecycleMessage::Init),
                },
                callback: None,
                callback_once: None,
                is_pending: false,
            }]),
            mq_buffer2: Rc::new(RefCell::new(Vec::new())),
            topic_subscriber: Rc::new(RefCell::new(HashMap::new())),
            msg_seq_inc: Rc::new(RefCell::new(0)),
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
                    topic_subscriber: self.topic_subscriber.clone(),
                    msg_seq_inc: self.msg_seq_inc.clone(),
                }),
                NodeName::Scheduler,
                MessageTo::Broadcast,
                MessageWithHeader { 
                    seq: 0, 
                    body: Message::Schedule,
                },
            );
        }

        // 消费消息
        for MessageQueueItem {
            from,
            to,
            message,
            mut callback_once,
            callback,
            is_pending,
        } in self.mq_buffer1.borrow_mut().drain(..)
        {
            if !is_pending {
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
                                topic_subscriber: self.topic_subscriber.clone(),
                                msg_seq_inc: self.msg_seq_inc.clone(),
                            }),
                            from,
                            to,
                            message.clone(),
                        );
                        debug!("handle message result: {ret:?}");
                        match ret {
                            HandleResult::Successful(e) => {
                                if let Some(cb) = callback_once.take() {
                                    cb(*node_name, HandleResult::Successful(e.clone()));
                                }
                                if let Some(ref cb) = callback {
                                    cb(*node_name, HandleResult::Successful(e));
                                }
                            }
                            HandleResult::Error(e) => {
                                if let Some(cb) = callback_once.take() {
                                    cb(*node_name, HandleResult::Error(e.clone()));
                                }
                                if let Some(ref cb) = callback {
                                    cb(*node_name, HandleResult::Error(e));
                                }
                            }
                            HandleResult::Pending => {
                                unimplemented!("broadcast is unsupported for pending message")
                            }
                            HandleResult::Discard => {}
                        }
                    }
                }
                MessageTo::Point(node_name) => {
                    self.nodes
                        .entry(node_name)
                        .and_modify(|x| {
                            if !is_pending {
                                debug!("handle message from node: {from:?}, to node: {node_name:?}, msg: {}", message.body.debug_msg());
                            }
                            let ret = x.handle_message(
                                Rc::new(ContextImpl {
                                    node_name,
                                    mq_buffer: self.mq_buffer2.clone(),
                                    topic_subscriber: self.topic_subscriber.clone(),
                                    msg_seq_inc: self.msg_seq_inc.clone(),
                                }),
                                from,
                                to,
                                message.clone(),
                            );
                            if !is_pending {
                                debug!("handle message result: {ret:?}");
                            }

                            match ret {
                                HandleResult::Successful(e) => {
                                    if let Some(cb) = callback_once.take() {
                                        cb(node_name, HandleResult::Successful(e.clone()));
                                    }
                                    if let Some(ref cb) = callback {
                                        cb(node_name, HandleResult::Successful(e));
                                    }
                                }
                                HandleResult::Error(e) => {
                                    if let Some(cb) = callback_once.take() {
                                        cb(node_name, HandleResult::Error(e.clone()));
                                    }
                                    if let Some(ref cb) = callback {
                                        cb(node_name, HandleResult::Error(e));
                                    }
                                }
                                HandleResult::Pending => { // 复制一份消息，下一轮pending将继续传递
                                    self.mq_buffer2.borrow_mut().push(MessageQueueItem { 
                                        from: from, 
                                        to: to, 
                                        message: message, 
                                        callback_once: callback_once, 
                                        callback: callback,
                                        is_pending: true,
                                    })
                                }
                                HandleResult::Discard => {}
                            }
                        })
                        .or_insert_with(|| {
                            // 如果不存在，则panic
                            panic!("not found node {:?}", node_name);
                        });
                }
                MessageTo::Topic(topic) => {
                    if let Some(nodes) = self.topic_subscriber.borrow().get(&topic) {
                        for node_name in nodes.iter() {
                            self.nodes.entry(*node_name).and_modify(|x| {
                                debug!(
                                    "handle message from node: {from:?}, to node: {node_name:?}, msg: {}",
                                    message.body.debug_msg()
                                );
                                let ret = x.handle_message(
                                    Rc::new(ContextImpl {
                                        node_name: *node_name,
                                        mq_buffer: self.mq_buffer2.clone(),
                                        topic_subscriber: self.topic_subscriber.clone(),
                                        msg_seq_inc: self.msg_seq_inc.clone(),
                                    }),
                                    from,
                                    to,
                                    message.clone(),
                                );
                                debug!("handle message result: {ret:?}");
                                
                                match ret {
                                    HandleResult::Successful(e) => {
                                        if let Some(cb) = callback_once.take() {
                                            cb(*node_name, HandleResult::Successful(e.clone()));
                                        }
                                        if let Some(ref cb) = callback {
                                            cb(*node_name, HandleResult::Successful(e));
                                        }
                                    }
                                    HandleResult::Error(e) => {
                                        if let Some(cb) = callback_once.take() {
                                            cb(*node_name, HandleResult::Error(e.clone()));
                                        }
                                        if let Some(ref cb) = callback {
                                            cb(*node_name, HandleResult::Error(e));
                                        }
                                    }
                                    HandleResult::Pending => {
                                        unimplemented!("topic is unsupported for pending message")
                                    }
                                    HandleResult::Discard => {}
                                }
                            }).or_insert_with(|| {
                                panic!("not found node {:?}", *node_name);
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
