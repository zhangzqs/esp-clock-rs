use std::{rc::Rc, time::Duration};

use log::info;
use slint::ComponentHandle;
use time::OffsetDateTime;

use crate::proto::{
    ipc, Context, HandleResult, LifecycleMessage, Message, MessageWithHeader, Node, NodeName,
    RoutePage, RouterMessage, WiFiMessage, WiFiStorageConfiguration,
};
use crate::{get_app_window, ui};

pub struct BootPage {
    t: slint::Timer,
}

impl BootPage {
    pub fn new() -> Self {
        Self {
            t: slint::Timer::default(),
        }
    }
}

impl BootPage {
    fn start_performance_monitor(&self, ctx: Rc<dyn Context>) {
        let p = ipc::PerformanceClient(ctx);
        self.t.start(
            slint::TimerMode::Repeated,
            Duration::from_secs(1),
            move || {
                if let Some(ui) = get_app_window().upgrade() {
                    let vm = ui.global::<ui::PerformanceViewModel>();
                    vm.set_largest_free_block(p.get_largeest_free_block() as i32);
                    vm.set_memory(p.get_free_heap_size() as i32);
                    vm.set_fps(p.get_fps() as i32);
                }
            },
        );
    }

    fn set_boot_time(&self, ctx: Rc<dyn Context>) {
        let t = ipc::TimestampClient(ctx.clone()).get_timestamp_nanos();
        let t = OffsetDateTime::from_unix_timestamp_nanos(t).unwrap();
        ipc::StorageClient(ctx.clone())
            .set("boot-time".into(), t.to_string().into())
            .unwrap();
    }

    fn connect_wifi(&self, ctx: Rc<dyn Context>) {
        let stg = ipc::StorageClient(ctx.clone());
        let ssid = stg.get("wifi/ssid".into()).unwrap().into();
        let password = stg.get("wifi/password".into()).unwrap().into();
        let ctx_ref = ctx.clone();
        ctx.async_call(
            NodeName::WiFi,
            Message::WiFi(WiFiMessage::ConnectRequest(WiFiStorageConfiguration {
                ssid,
                password: Some(password),
            })),
            Box::new(move |r| {
                // wifi连接成功
                info!("wifi连接完成, 跳转路由: {r:?}");
                ctx_ref.sync_call(
                    NodeName::Router,
                    Message::Router(RouterMessage::GotoPage(RoutePage::Home)),
                );
            }),
        );
    }

    fn init(&self, ctx: Rc<dyn Context>) {
        self.set_boot_time(ctx.clone());
        self.start_performance_monitor(ctx.clone());
        self.connect_wifi(ctx.clone());
    }
}

impl Node for BootPage {
    fn node_name(&self) -> NodeName {
        NodeName::BootPage
    }

    fn handle_message(&self, ctx: Rc<dyn Context>, msg: MessageWithHeader) -> HandleResult {
        match msg.body {
            Message::Lifecycle(msg) => match msg {
                LifecycleMessage::Init => {
                    self.init(ctx.clone());
                    return HandleResult::Finish(Message::Empty);
                }
                _ => {}
            },
            _ => {}
        }
        HandleResult::Discard
    }
}
