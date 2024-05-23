use std::{rc::Rc, time::Duration};

use log::info;
use slint::{ComponentHandle, Weak};
use time::OffsetDateTime;

use crate::proto::{
    ipc, Context, HandleResult, LifecycleMessage, Message, MessageTo, MessageWithHeader, Node,
    NodeName, RoutePage, RouterMessage, TimerMessage,
};
use crate::ui::{self, AppWindow};

pub struct BootPage {
    app: Weak<AppWindow>,
    t: slint::Timer,
}

impl BootPage {
    pub fn new(app: Weak<AppWindow>) -> Self {
        Self {
            app,
            t: slint::Timer::default(),
        }
    }
}

impl Node for BootPage {
    fn node_name(&self) -> NodeName {
        NodeName::BootPage
    }

    fn handle_message(
        &self,
        ctx: Rc<dyn Context>,
        _from: NodeName,
        _to: MessageTo,
        msg: MessageWithHeader,
    ) -> HandleResult {
        match msg.body {
            Message::Lifecycle(msg) => match msg {
                LifecycleMessage::Init => {
                    let ctx_ref = ctx.clone();
                    slint::Timer::single_shot(Duration::from_secs(1), move || {
                        ctx_ref.clone().sync_call(
                            NodeName::Router,
                            Message::Router(RouterMessage::GotoPage(RoutePage::Home)),
                        );
                    });

                    let ctx_ref = ctx.clone();
                    ipc::TimestampClient(ctx.clone()).get_timestamp_nanos(Box::new(move |t| {
                        let t = OffsetDateTime::from_unix_timestamp_nanos(t).unwrap();
                        ipc::StorageClient(ctx_ref.clone()).set(
                            "boot-time".into(),
                            Some(t.to_string()),
                            Box::new(|_| {}),
                        );
                    }));

                    let ctx_ref = ctx.clone();
                    let app = self.app.clone();
                    self.t.start(
                        slint::TimerMode::Repeated,
                        Duration::from_secs(1),
                        move || {
                            let ctx_ref = ctx_ref.clone();
                            let p = ipc::PerformanceClient(ctx_ref);

                            let app_ref = app.clone();
                            p.get_largeest_free_block(Box::new(move |x| {
                                if let Some(ui) = app_ref.upgrade() {
                                    ui.global::<ui::PerformanceViewModel>()
                                        .set_largest_free_block(x as i32);
                                }
                            }));
                            let app_ref = app.clone();
                            p.get_free_heap_size(Box::new(move |x| {
                                if let Some(ui) = app_ref.upgrade() {
                                    ui.global::<ui::PerformanceViewModel>().set_memory(x as i32);
                                }
                            }));
                            let app_ref = app.clone();
                            p.get_fps(Box::new(move |x| {
                                if let Some(ui) = app_ref.upgrade() {
                                    ui.global::<ui::PerformanceViewModel>().set_fps(x as i32);
                                }
                            }));
                        },
                    );

                    return HandleResult::Finish(Message::Empty);
                }
                _ => {}
            },
            _ => {}
        }
        HandleResult::Discard
    }
}
