use std::rc::Rc;

use dev::DevConfigSetter;
use node::*;

mod adapter;
mod node;
mod scheduler;
mod ui;

pub use proto;
pub use proto::storage;
pub use scheduler::Scheduler;
pub use ui::get_app_window;

static mut SCHEDULER: Option<Rc<Scheduler>> = None;

pub fn get_scheduler() -> Rc<Scheduler> {
    unsafe {
        SCHEDULER
            .get_or_insert_with(|| {
                let s = Scheduler::new();
                register_default_nodes(&s);
                Rc::new(s)
            })
            .clone()
    }
}

fn register_default_nodes(sche: &Scheduler) {
    sche.register_node(HomePage::new());
    sche.register_node(WeatherPage::new());
    sche.register_node(MenuPage::new());
    sche.register_node(BootPage::new());
    sche.register_node(AlertDialog::new());
    sche.register_node(MusicPage::new());

    sche.register_node(RouterService::new());
    sche.register_node(TouchOneButtonAdapterService::new());
    sche.register_node(WeatherService::new());
    sche.register_node(MockStorageService::new());
    sche.register_node(MockSystemService {});
    sche.register_node(TimerService::new());

    // #[cfg(dev_config)]
    sche.register_node(DevConfigSetter {});

    sche.register_node(MockWiFiService::new());
    sche.register_node(MidiPlayerService::new());
    sche.register_node(CanvasView::new());
    sche.register_node(UserAlarmService::new());
}

mod dev {
    use std::collections::HashMap;

    use super::proto::*;

    pub struct DevConfigSetter {}

    impl Node for DevConfigSetter {
        fn priority(&self) -> usize {
            999
        }
        fn node_name(&self) -> NodeName {
            NodeName::Other("DevConfigSetter".into())
        }

        fn handle_message(
            &self,
            ctx: std::rc::Rc<dyn Context>,
            msg: MessageWithHeader,
        ) -> HandleResult {
            match msg.body {
                Message::Lifecycle(LifecycleMessage::Init) => {
                    let cfg = serde_json::from_slice::<HashMap<String, String>>(include_bytes!(
                        "../config.json"
                    ))
                    .unwrap();
                    let stg = ipc::StorageClient(ctx);
                    for (k, v) in cfg.into_iter() {
                        stg.set(k, StorageValue::String(v)).unwrap();
                    }
                }
                _ => {}
            }
            HandleResult::Discard
        }
    }
}
