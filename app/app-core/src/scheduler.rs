use std::{
    cell::RefCell,
    collections::HashMap,
    rc::Rc,
    sync::atomic::{AtomicU32, Ordering},
};

use futures_util::{
    future::{BoxFuture, LocalBoxFuture},
    Future, FutureExt,
};
use log::debug;

use crate::proto::*;

struct MessageQueueItem {
    from: NodeName,
    to: MessageTo,
    message: MessageWithHeader,
    callback_once: Option<MessageCallbackOnce>,
}

struct FutureImpl {
    result: Rc<RefCell<Option<HandleResult>>>,
}

impl Future for FutureImpl {
    type Output = HandleResult;

    fn poll(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Self::Output> {
        if let Some(x) = self.result.borrow_mut().take() {
            std::task::Poll::Ready(x)
        } else {
            std::task::Poll::Pending
        }
    }
}

struct ContextImpl {
    node_name: NodeName,
    mq_buffer: Rc<RefCell<Vec<MessageQueueItem>>>,
    msg_seq_inc: AtomicU32,
    nodes: Rc<RefCell<HashMap<NodeName, Box<dyn Node>>>>,
}

impl Context for ContextImpl {
    // 发送广播消息
    fn boardcast(&self, msg: Message) {
        self.msg_seq_inc.fetch_add(1, Ordering::SeqCst);
        self.mq_buffer.borrow_mut().push(MessageQueueItem {
            from: self.node_name,
            to: MessageTo::Broadcast,
            message: MessageWithHeader {
                seq: self.msg_seq_inc.load(Ordering::SeqCst),
                body: msg,
                is_pending: false,
            },
            callback_once: None,
        })
    }

    // 异步调用
    fn async_call(&self, node: NodeName, msg: Message, callback: MessageCallbackOnce) {
        self.msg_seq_inc.fetch_add(1, Ordering::SeqCst);
        self.mq_buffer.borrow_mut().push(MessageQueueItem {
            from: self.node_name,
            to: MessageTo::Point(node),
            message: MessageWithHeader {
                seq: self.msg_seq_inc.load(Ordering::SeqCst),
                body: msg,
                is_pending: false,
            },
            callback_once: Some(callback),
        })
    }

    // 同步调用
    fn sync_call(&self, node: NodeName, msg: Message) -> HandleResult {
        self.msg_seq_inc.fetch_add(1, Ordering::SeqCst);
        self.nodes.borrow()[&node].handle_message(
            Rc::new(ContextImpl {
                node_name: self.node_name,
                mq_buffer: self.mq_buffer.clone(),
                nodes: self.nodes.clone(),
                msg_seq_inc: AtomicU32::new(self.msg_seq_inc.load(Ordering::SeqCst)),
            }),
            self.node_name,
            MessageTo::Point(node),
            MessageWithHeader {
                seq: self.msg_seq_inc.load(Ordering::SeqCst),
                is_pending: false,
                body: msg,
            },
        )
    }
}

pub struct Scheduler {
    nodes: Rc<RefCell<HashMap<NodeName, Box<dyn Node>>>>,
    mq_buffer1: RefCell<Vec<MessageQueueItem>>,
    mq_buffer2: Rc<RefCell<Vec<MessageQueueItem>>>,
    msg_seq_inc: AtomicU32,
}

impl Default for Scheduler {
    fn default() -> Self {
        Self::new()
    }
}

impl Scheduler {
    pub fn new() -> Self {
        Self {
            nodes: Rc::new(RefCell::new(HashMap::new())),
            mq_buffer1: RefCell::new(vec![MessageQueueItem {
                from: NodeName::Scheduler,
                to: MessageTo::Broadcast,
                message: MessageWithHeader {
                    seq: 0,
                    body: Message::Lifecycle(LifecycleMessage::Init),
                    is_pending: false,
                },
                callback_once: None,
            }]),
            mq_buffer2: Rc::new(RefCell::new(Vec::new())),
            msg_seq_inc: AtomicU32::new(0),
        }
    }

    pub fn register_node<A: Node + 'static>(&self, app: A) {
        self.nodes
            .borrow_mut()
            .insert(app.node_name(), Box::new(app));
    }

    pub fn schedule_once(&self) {
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
                    for (node_name, node) in self.nodes.borrow().iter() {
                        debug!(
                            "handle message from node: {from:?}, to node: {node_name:?}, msg: {}",
                            message.body.debug_msg()
                        );
                        let ret = node.handle_message(
                            Rc::new(ContextImpl {
                                node_name: *node_name,
                                mq_buffer: self.mq_buffer2.clone(),
                                msg_seq_inc: AtomicU32::new(
                                    self.msg_seq_inc.load(Ordering::SeqCst),
                                ),
                                nodes: self.nodes.clone(),
                            }),
                            from,
                            to,
                            message.clone(),
                        );
                        debug!("handle message result: {ret:?}");
                        match ret {
                            HandleResult::Finish(_) => {
                                if callback_once.take().is_some() {
                                    unimplemented!("broadcast is unsupported for callback")
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
                    if !self.nodes.borrow().contains_key(&node_name) {
                        panic!("not found node {:?}", node_name);
                    }
                    if !message.is_pending {
                        debug!(
                            "handle message from node: {from:?}, to node: {node_name:?}, msg: {}",
                            message.body.debug_msg()
                        );
                    }
                    let ret = self.nodes.borrow()[&node_name].handle_message(
                        Rc::new(ContextImpl {
                            node_name,
                            mq_buffer: self.mq_buffer2.clone(),
                            msg_seq_inc: AtomicU32::new(self.msg_seq_inc.load(Ordering::SeqCst)),
                            nodes: self.nodes.clone(),
                        }),
                        from,
                        to,
                        message.clone(),
                    );
                    if !message.is_pending {
                        debug!("handle message result: {ret:?}");
                    }

                    match ret {
                        HandleResult::Finish(e) => {
                            if let Some(cb) = callback_once.take() {
                                cb(HandleResult::Finish(e.clone()));
                            }
                        }
                        HandleResult::Pending => {
                            // 复制一份消息，下一轮pending将继续传递
                            let mut message = message;
                            message.is_pending = true;
                            self.mq_buffer2.borrow_mut().push(MessageQueueItem {
                                from,
                                to,
                                message,
                                callback_once,
                            });
                        }
                        HandleResult::Discard => {}
                    }
                }
            }
        }
        // 交换两个缓冲区队列
        self.mq_buffer2.swap(&self.mq_buffer1)
    }
}
