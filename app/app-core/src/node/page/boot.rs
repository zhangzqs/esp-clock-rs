use std::cell::RefCell;
use std::{rc::Rc, time::Duration};

use log::{error, info};
use proto::TopicName;
use slint::ComponentHandle;
use std::fmt::Debug;
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
        // 幂等性
        if self.t.borrow().is_some() {
            return;
        }
        let p = ipc::SystemClient(ctx);
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

    fn stop_performance_monitor(&self) {
        // 幂等性
        if self.t.borrow().is_none() {
            return;
        }
        if let Some(ui) = get_app_window().upgrade() {
            let vm = ui.global::<ui::PerformanceViewModel>();
            vm.set_is_show(false);
        }
        self.t.take();
    }

    fn set_boot_time(&self, ctx: Rc<dyn Context>) {
        ipc::StorageClient(ctx.clone())
            .set(
                "boot-time".into(),
                OffsetDateTime::now_utc().to_string().into(),
            )
            .unwrap();
    }

    fn alert_dialog<T: Debug>(ctx: Rc<dyn Context>, e: T) {
        error!("error: {e:?}");
        ctx.async_call(
            NodeName::AlertDialog,
            Message::AlertDialog(AlertDialogMessage::ShowRequest {
                duration: Some(3000),
                content: AlertDialogContent {
                    text: Some(format!("{e:?}")),
                    image: None,
                },
            }),
            Box::new(|_| {}),
        )
    }

    fn connect_wifi(&self, ctx: Rc<dyn Context>) {
        let stg = WiFiStorage(ipc::StorageClient(ctx.clone()));
        if let Some(ssid) = stg.get_ssid() {
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
        } else {
            ctx.clone().async_call(
                NodeName::WiFi,
                Message::WiFi(WiFiMessage::StartAPRequest),
                Box::new(move |r| {
                    match &r {
                        HandleResult::Finish(Message::WiFi(msg)) => match msg {
                            WiFiMessage::StartAPResponse => {
                                ctx.clone().async_call(
                                    NodeName::WiFi,
                                    Message::WiFi(WiFiMessage::GetIpInfoRequest),
                                    Box::new(move |r| {
                                        match &r {
                                            HandleResult::Finish(Message::WiFi(
                                                WiFiMessage::GetIpInfoResponse(netinfo),
                                            )) => {
                                                ctx.clone().async_call(
                                                    NodeName::AlertDialog,
                                                    Message::AlertDialog(
                                                        AlertDialogMessage::ShowRequest {
                                                            duration: None,
                                                            content: AlertDialogContent {
                                                                text: Some(format!("Please connect to AP \"ESP-CLOCK-RS\" and open \"http://{}\" to config wifi then click button to restart.", netinfo.ip)),
                                                                image: None,
                                                            },
                                                        },
                                                    ),
                                                    Box::new(move |_| {
                                                        ctx.clone().sync_call(NodeName::System, Message::System(SystemMessage::Restart));
                                                    }),
                                                );
                                                return;
                                            },
                                            _ => {}
                                        }
                                        panic!("unexpected response {r:?}")
                                    }),
                                );
                                return;
                            }
                            WiFiMessage::Error(e) => {
                                Self::alert_dialog(ctx.clone(), e);
                                return;
                            }
                            _ => {}
                        },
                        _ => {}
                    }
                    panic!("unexpected response {r:?}")
                }),
            );
        }
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
            Message::BootPage(BootPageMessage::EnableSystemMonitor(enable)) => {
                if enable {
                    self.start_performance_monitor(ctx.clone());
                } else {
                    self.stop_performance_monitor();
                }
            }
            _ => {}
        }
        HandleResult::Discard
    }
}
