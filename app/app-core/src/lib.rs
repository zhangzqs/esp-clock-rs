use dev::DevConfigSetter;
use node::*;
pub use scheduler::Scheduler;

mod adapter;
mod node;
pub mod proto;
mod scheduler;
mod ui;

pub use scheduler::get_scheduler;
pub use ui::get_app_window;

pub fn register_default_nodes(sche: &Scheduler) {
    sche.register_node(HomePage::new());
    sche.register_node(WeatherPage::new());
    sche.register_node(MenuPage::new());
    sche.register_node(BootPage::new());

    sche.register_node(RouterService::new());
    sche.register_node(TouchOneButtonAdapterService::new());
    sche.register_node(DefaultTimestampService {});
    sche.register_node(WeatherService::new());
    sche.register_node(MockStorageService::new());
    sche.register_node(MockPerformanceService {});
    sche.register_node(TimerService::new());
    sche.register_node(DevConfigSetter {})
}

mod dev {
    use std::collections::HashMap;

    use super::proto::*;

    pub struct DevConfigSetter {}

    impl Node for DevConfigSetter {
        fn node_name(&self) -> NodeName {
            NodeName::Other("DevConfigSetter")
        }

        fn handle_message(
            &self,
            ctx: std::rc::Rc<dyn Context>,
            _from: NodeName,
            _to: MessageTo,
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
                        stg.set(k, Some(v)).unwrap();
                    }
                }
                _ => {}
            }
            HandleResult::Discard
        }
    }
}
