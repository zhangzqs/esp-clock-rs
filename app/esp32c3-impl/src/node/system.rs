use std::{
    rc::Rc,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    time::Duration,
};

use app_core::proto::*;
use esp_idf_sys as _;

pub struct SystemService {
    timer: slint::Timer,
    frame_counter: Arc<AtomicUsize>,
    fps: Arc<AtomicUsize>,
}

impl SystemService {
    pub fn new(frame_counter: Arc<AtomicUsize>) -> Self {
        Self {
            frame_counter,
            timer: slint::Timer::default(),
            fps: Arc::new(AtomicUsize::new(0)),
        }
    }
}

impl Node for SystemService {
    fn node_name(&self) -> NodeName {
        NodeName::System
    }

    fn handle_message(&self, _ctx: Rc<dyn Context>, msg: MessageWithHeader) -> HandleResult {
        match msg.body {
            Message::Lifecycle(LifecycleMessage::Init) => {
                let frame_counter = self.frame_counter.clone();
                let fps = self.fps.clone();
                self.timer.start(
                    slint::TimerMode::Repeated,
                    Duration::from_secs(1),
                    move || {
                        fps.store(frame_counter.load(Ordering::SeqCst), Ordering::SeqCst);
                        frame_counter.store(0, Ordering::SeqCst);
                    },
                );
            }
            Message::System(pm) => {
                return HandleResult::Finish(Message::System(match pm {
                    SystemMessage::GetFreeHeapSizeRequest => {
                        SystemMessage::GetFreeHeapSizeResponse(unsafe {
                            esp_idf_sys::esp_get_free_heap_size() as usize
                        })
                    }
                    SystemMessage::GetLargestFreeBlock => {
                        SystemMessage::GetLargestFreeBlockResponse(unsafe {
                            esp_idf_sys::heap_caps_get_largest_free_block(
                                esp_idf_sys::MALLOC_CAP_8BIT,
                            )
                        })
                    }
                    SystemMessage::GetFpsRequest => {
                        let fps = self.fps.load(Ordering::SeqCst);
                        SystemMessage::GetFpsResponse(fps)
                    }
                    SystemMessage::Restart => unsafe {
                        esp_idf_sys::esp_restart();
                    },
                    m => panic!("unexpected message {m:?}"),
                }));
            }
            _ => {}
        }
        HandleResult::Discard
    }
}
