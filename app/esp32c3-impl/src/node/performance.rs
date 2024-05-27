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

pub struct PerformanceService {
    timer: slint::Timer,
    frame_counter: Arc<AtomicUsize>,
    fps: Arc<AtomicUsize>,
}

impl PerformanceService {
    pub fn new(frame_counter: Arc<AtomicUsize>) -> Self {
        Self {
            frame_counter,
            timer: slint::Timer::default(),
            fps: Arc::new(AtomicUsize::new(0)),
        }
    }
}

impl Node for PerformanceService {
    fn node_name(&self) -> NodeName {
        NodeName::Performance
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
            Message::Performance(pm) => {
                return HandleResult::Finish(Message::Performance(match pm {
                    PerformanceMessage::GetFreeHeapSizeRequest => {
                        PerformanceMessage::GetFreeHeapSizeResponse(unsafe {
                            esp_idf_sys::esp_get_free_heap_size() as usize
                        })
                    }
                    PerformanceMessage::GetLargestFreeBlock => {
                        PerformanceMessage::GetLargestFreeBlockResponse(unsafe {
                            esp_idf_sys::heap_caps_get_largest_free_block(
                                esp_idf_sys::MALLOC_CAP_8BIT,
                            )
                        })
                    }
                    PerformanceMessage::GetFpsRequest => {
                        let fps = self.fps.load(Ordering::SeqCst);
                        PerformanceMessage::GetFpsResponse(fps)
                    }
                    m => panic!("unexpected message {m:?}"),
                }));
            }
            _ => {}
        }
        HandleResult::Discard
    }
}
