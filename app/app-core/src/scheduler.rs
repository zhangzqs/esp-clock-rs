use std::{
    cell::RefCell,
    collections::{HashMap, VecDeque},
    rc::Rc,
    sync::atomic::{AtomicUsize, Ordering},
};

use log::{error, info};

use crate::proto::*;

/// 全局消息计数器
static MSG_SEQ_COUNT: AtomicUsize = AtomicUsize::new(0);

fn gen_msg_seq() -> usize {
    MSG_SEQ_COUNT.fetch_add(1, Ordering::SeqCst);
    MSG_SEQ_COUNT.load(Ordering::SeqCst)
}

#[derive(Default)]
struct WaitGroupImpl {
    counter: AtomicUsize,
    callback: RefCell<Option<Box<dyn FnOnce()>>>,
}

impl WaitGroupImpl {
    pub fn handle(&self) -> bool {
        let is_done = self.counter.load(Ordering::SeqCst) == 0;
        if is_done {
            if let Some(f) = self.callback.borrow_mut().take() {
                f();
            }
        }
        is_done
    }
}

impl WaitGroup for WaitGroupImpl {
    fn inc(&self) {
        self.counter.fetch_add(1, Ordering::SeqCst);
    }

    fn done(&self) {
        self.counter.fetch_sub(1, Ordering::SeqCst);
    }

    fn wait(&self, callback: Box<dyn FnOnce()>) {
        *self.callback.borrow_mut() = Some(callback);
    }
}

struct MessageQueueItem {
    message: MessageWithHeader,
    /// 异步消息是否处于pending态
    is_pending: bool,
    callback_once: Option<MessageCallbackOnce>,
}

struct ContextImpl {
    node_name: NodeName,
    mq_buffer: Rc<RefCell<Vec<MessageQueueItem>>>,
    nodes: Rc<RefCell<HashMap<NodeName, Box<dyn Node>>>>,
    ready_result: Rc<RefCell<HashMap<usize, Message>>>,
    subscriber: Rc<RefCell<HashMap<TopicName, VecDeque<NodeName>>>>,
    wg_queue: Rc<RefCell<Vec<Rc<WaitGroupImpl>>>>,
}

impl Context for ContextImpl {
    // 发送全局广播消息
    fn broadcast_global(&self, msg: Message) {
        self.mq_buffer.borrow_mut().push(MessageQueueItem {
            message: MessageWithHeader {
                from: self.node_name.clone(),
                to: MessageTo::Broadcast,
                seq: gen_msg_seq(),
                body: msg,
            },
            is_pending: false,
            callback_once: None,
        })
    }

    // 发送话题广播消息
    fn broadcast_topic(&self, topic: TopicName, msg: Message) {
        self.mq_buffer.borrow_mut().push(MessageQueueItem {
            message: MessageWithHeader {
                from: self.node_name.clone(),
                to: MessageTo::Topic(topic),
                seq: gen_msg_seq(),
                body: msg,
            },
            is_pending: false,
            callback_once: None,
        })
    }

    // 订阅话题消息
    fn subscribe_topic(&self, topic: TopicName) {
        info!("node {:?} subscribe topic {:?}", self.node_name, topic);

        self.subscriber
            .borrow_mut()
            .entry(topic.clone())
            .or_default();

        // 形成一个栈，后订阅的先收到消息，方便形成事件冒泡，利用HandleResult::Block机制阻断广播
        // 较近的订阅者可以阻断其他订阅者接收广播
        // TODO: 数据结构性能优化
        let node = &self.node_name;
        self.subscriber.borrow_mut().entry(topic).and_modify(|vec| {
            if vec.contains(node) {
                vec.retain(|x| x != node);
            }
            vec.push_front(node.clone());
        });
    }

    // 解除订阅话题
    fn unsubscribe_topic(&self, topic: TopicName) {
        info!("node {:?} unsubscribe topic {:?}", self.node_name, topic);

        let node = &self.node_name;
        self.subscriber.borrow_mut().entry(topic).and_modify(|x| {
            x.retain(|x| x != node);
        });
    }

    // 异步调用
    fn async_call(&self, node: NodeName, msg: Message, callback: MessageCallbackOnce) {
        // 目标不存在
        if !self.nodes.borrow().contains_key(&node) {
            error!("not found node {:?}", node);
            callback(HandleResult::Discard);
            return;
        }
        self.mq_buffer.borrow_mut().push(MessageQueueItem {
            message: MessageWithHeader {
                from: self.node_name.clone(),
                to: MessageTo::Point(node),
                seq: gen_msg_seq(),
                body: msg,
            },
            is_pending: false,
            callback_once: Some(callback),
        })
    }

    // 同步调用
    fn sync_call(&self, node: NodeName, msg: Message) -> HandleResult {
        // 目标不存在
        if !self.nodes.borrow().contains_key(&node) {
            error!("not found node {:?}", node);
            return HandleResult::Discard;
        }
        let msg = MessageWithHeader {
            from: self.node_name.clone(),
            to: MessageTo::Point(node.clone()),
            body: msg,
            seq: gen_msg_seq(),
        };
        // info!("dispatch sync p2p message {:?}", msg);
        let ret = self.nodes.borrow()[&node].handle_message(
            Rc::new(ContextImpl {
                node_name: node,
                mq_buffer: self.mq_buffer.clone(),
                nodes: self.nodes.clone(),
                ready_result: self.ready_result.clone(),
                subscriber: self.subscriber.clone(),
                wg_queue: self.wg_queue.clone(),
            }),
            msg,
        );
        // info!("handle sync p2p message result: {:?}", ret);
        ret
    }

    // 异步结果就绪
    fn async_ready(&self, seq: usize, result: Message) {
        self.ready_result.borrow_mut().insert(seq, result);
    }

    fn create_wait_group(&self) -> Rc<dyn WaitGroup> {
        let wg = Rc::new(WaitGroupImpl::default());
        self.wg_queue.borrow_mut().push(wg.clone());
        wg.clone()
    }
}

#[derive(Default)]
pub struct Scheduler {
    broadcast_order: RefCell<Vec<NodeName>>,
    nodes: Rc<RefCell<HashMap<NodeName, Box<dyn Node>>>>,
    mq_buffer1: RefCell<Vec<MessageQueueItem>>,
    mq_buffer2: Rc<RefCell<Vec<MessageQueueItem>>>,
    ready_result: Rc<RefCell<HashMap<usize, Message>>>,
    subscriber: Rc<RefCell<HashMap<TopicName, VecDeque<NodeName>>>>,
    wg_queue1: RefCell<Vec<Rc<WaitGroupImpl>>>,
    wg_queue2: Rc<RefCell<Vec<Rc<WaitGroupImpl>>>>,
}

impl Scheduler {
    pub(crate) fn new() -> Self {
        let s = Self::default();
        s.mq_buffer1.borrow_mut().push(MessageQueueItem {
            message: MessageWithHeader {
                from: NodeName::Scheduler,
                to: MessageTo::Broadcast,
                seq: 0,
                body: Message::Lifecycle(LifecycleMessage::Init),
            },
            is_pending: false,
            callback_once: None,
        });
        s
    }

    pub fn register_node<A: Node + 'static>(&self, app: A) {
        info!("register node: {:?}", app.node_name());
        self.nodes
            .borrow_mut()
            .insert(app.node_name(), Box::new(app));
        // TODO: 使用优先队列优化数据结构
        let mut broadcast_order = self
            .nodes
            .borrow()
            .iter()
            .map(|(n, x)| (n.clone(), x.priority()))
            .collect::<Vec<_>>();
        broadcast_order.sort_by(|(_, ap), (_, bp)| bp.cmp(ap));
        let broadcast_order = broadcast_order.into_iter().map(|(n, _)| n).collect();
        *self.broadcast_order.borrow_mut() = broadcast_order;
        info!("broadcast_order: {:?}", self.broadcast_order.borrow());
    }

    fn gen_ctx(&self, node_name: &NodeName) -> Rc<dyn Context> {
        Rc::new(ContextImpl {
            node_name: node_name.clone(),
            mq_buffer: self.mq_buffer2.clone(),
            nodes: self.nodes.clone(),
            ready_result: self.ready_result.clone(),
            subscriber: self.subscriber.clone(),
            wg_queue: self.wg_queue2.clone(),
        })
    }

    fn broadcast_scheduler_heartbeat(&self) {
        self.broadcast_topic_message(
            &TopicName::Scheduler,
            MessageWithHeader {
                from: NodeName::Scheduler,
                to: MessageTo::Topic(TopicName::Scheduler),
                seq: 0,
                body: Message::Empty,
            },
        );
    }

    fn broadcast_message<'a, Iter: Iterator<Item = &'a NodeName>>(
        &self,
        node: Iter,
        message: MessageWithHeader,
    ) {
        let msg_only_header = MessageWithHeader {
            seq: message.seq,
            from: message.from.clone(),
            to: message.to.clone(),
            body: Default::default(),
        };
        for node_name in node {
            match self.nodes.borrow()[node_name]
                .handle_message(self.gen_ctx(node_name), message.clone())
            {
                HandleResult::Finish(_) => {
                    // 目前广播消息不处理Finish状态
                }
                HandleResult::Pending => {
                    // 消息没有就绪结果，改写为单点通信，标记is_pending后，继续排队到异步队列
                    self.mq_buffer2.borrow_mut().push(MessageQueueItem {
                        message: msg_only_header.clone(), // poll的时候不需要clone完整的消息body
                        is_pending: true,
                        callback_once: None,
                    });
                }
                HandleResult::Discard => {}
                HandleResult::Block => {
                    return;
                }
            }
        }
    }

    fn broadcast_global_message(&self, message: MessageWithHeader) {
        self.broadcast_message(self.broadcast_order.borrow().iter(), message);
    }

    fn broadcast_topic_message(&self, topic: &TopicName, message: MessageWithHeader) {
        if !self.subscriber.borrow().contains_key(topic) {
            return;
        }

        // 广播消息的处理函数中可能需要解除订阅，这将会对subscriber造成一次可变借用，故此处复制一份
        let cloned_subscriber = self.subscriber.borrow()[topic].clone();
        self.broadcast_message(cloned_subscriber.iter(), message);
    }

    fn handle_point_message(&self, node_name: &NodeName, mq_item: MessageQueueItem) {
        let message = mq_item.message;
        let mut callback_once = mq_item.callback_once;

        if mq_item.is_pending {
            // 轮询消息结果
            self.nodes.borrow()[node_name].poll(self.gen_ctx(node_name), message.seq);
            // 若轮询到了结果
            if let Some(m) = {
                let m = self.ready_result.borrow_mut().remove(&message.seq);
                m
            } {
                info!("async message seq {} is ready: {:?}", message.seq, m);
                // 如果消息已经就绪，则触发回调
                if let Some(cb) = callback_once.take() {
                    cb(HandleResult::Finish(m));
                }
            } else {
                // 若仍无消息结果就绪，则继续将进入消息队列排队轮询
                self.mq_buffer2.borrow_mut().push(MessageQueueItem {
                    message,
                    is_pending: true,
                    callback_once,
                });
            }
        } else {
            let msg_only_header = MessageWithHeader {
                seq: message.seq,
                from: message.from.clone(),
                to: message.to.clone(),
                body: Default::default(),
            };

            info!("dispatch async p2p message {:?}", message);
            let ret =
                self.nodes.borrow()[node_name].handle_message(self.gen_ctx(node_name), message);
            info!("handle async p2p message result: {:?}", ret);
            match ret {
                HandleResult::Finish(e) => {
                    if let Some(cb) = callback_once.take() {
                        cb(HandleResult::Finish(e));
                    }
                }
                HandleResult::Pending => {
                    // 消息没有就绪结果，标记is_pending后，继续排队到异步队列
                    self.mq_buffer2.borrow_mut().push(MessageQueueItem {
                        message: msg_only_header,
                        is_pending: true,
                        callback_once,
                    });
                }
                x => {
                    if let Some(cb) = callback_once.take() {
                        cb(x);
                    }
                }
            }
        }
    }

    pub fn schedule_once(&self) {
        // 广播心跳
        self.broadcast_scheduler_heartbeat();

        // 从mq1消费消息
        for item in self.mq_buffer1.borrow_mut().drain(..) {
            match item.message.to.clone() {
                MessageTo::Broadcast => self.broadcast_global_message(item.message),
                MessageTo::Topic(topic) => self.broadcast_topic_message(&topic, item.message),
                MessageTo::Point(node_name) => self.handle_point_message(&node_name, item),
            }
        }
        // 交换两个缓冲区队列，相当于mq2的消息移动到mq1
        self.mq_buffer2.swap(&self.mq_buffer1);

        // 检查wg是否done
        for wg in self.wg_queue1.borrow_mut().drain(..) {
            // 若wg还没有done，则流传到wg_queue2
            if !wg.handle() {
                self.wg_queue2.borrow_mut().push(wg);
            }
        }
        self.wg_queue2.swap(&self.wg_queue1);
    }
}
