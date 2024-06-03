use std::cell::RefCell;
use std::{rc::Rc, time::Duration};

use log::info;
use proto::TopicName;
use slint::ComponentHandle;
use time::OffsetDateTime;

use crate::proto::*;
use crate::storage::WiFiStorage;
use crate::{get_app_window, ui};

pub struct BootPage {
    t: RefCell<Option<slint::Timer>>,
}

impl BootPage {
    pub fn new() -> Self {
        Self {
            t: RefCell::new(None),
        }
    }
}

impl BootPage {
    fn start_performance_monitor(&self, ctx: Rc<dyn Context>) {
        let p = ipc::PerformanceClient(ctx);
        self.t
            .borrow_mut()
            .get_or_insert_with(slint::Timer::default)
            .start(
                slint::TimerMode::Repeated,
                Duration::from_secs(1),
                move || {
                    if let Some(ui) = get_app_window().upgrade() {
                        let vm = ui.global::<ui::PerformanceViewModel>();
                        vm.set_is_show(true);
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
        let stg = WiFiStorage(ipc::StorageClient(ctx.clone()));
        let ssid = stg.get_ssid().unwrap_or_default();
        let password = stg.get_password().unwrap_or_default();
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

    fn animate(&self) {
        if let Some(ui) = get_app_window().upgrade() {
            let vm = ui.global::<ui::BootPageViewModel>();
            vm.invoke_play_mihoyo();
        }
        slint::Timer::single_shot(Duration::from_secs(3), || {
            if let Some(ui) = get_app_window().upgrade() {
                let vm = ui.global::<ui::BootPageViewModel>();
                vm.invoke_play_genshin();
            }
            slint::Timer::single_shot(Duration::from_secs(3), || {
                if let Some(ui) = get_app_window().upgrade() {
                    let vm = ui.global::<ui::BootPageViewModel>();
                    vm.invoke_play_gate();
                }
            });
        });
    }

    fn init(&self, ctx: Rc<dyn Context>) {
        self.start_performance_monitor(ctx.clone());
        self.animate();
        self.set_boot_time(ctx.clone());
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
                    ctx.subscribe_topic(TopicName::OneButton);
                    self.init(ctx.clone());
                    return HandleResult::Finish(Message::Empty);
                }
                LifecycleMessage::Show => {
                    // 首屏组件，默认直接显示，没有其他页面会发送这个消息
                    // 故按键订阅需要在init消息处完成
                    return HandleResult::Finish(Message::Empty);
                }
                LifecycleMessage::Hide => {
                    ctx.unsubscribe_topic(TopicName::OneButton);
                    return HandleResult::Finish(Message::Empty);
                }
            },
            Message::OneButton(proto::OneButtonMessage::Clicks(2)) => {
                self.start_performance_monitor(ctx.clone());
                return HandleResult::Finish(Message::Empty);
            }
            _ => {}
        }
        HandleResult::Discard
    }
}
