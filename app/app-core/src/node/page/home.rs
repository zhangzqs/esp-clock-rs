use crate::proto::{
    ipc, Context, HandleResult, LifecycleMessage, Message, MessageTo, MessageWithHeader, Node,
    NodeName, OneButtonMessage, RoutePage, RouterMessage,
};
use crate::ui::{AppWindow, HomeViewModel, TimeData};
use slint::{ComponentHandle, Weak};
use std::{rc::Rc, time::Duration};
use time::{OffsetDateTime, UtcOffset};

pub struct HomePage {
    app: Weak<AppWindow>,
    time_update_timer: Option<slint::Timer>,
    is_show: bool,
}

impl HomePage {
    pub fn new(app: Weak<AppWindow>) -> Self {
        Self {
            app,
            time_update_timer: None,
            is_show: false,
        }
    }
}

impl HomePage {
    fn update_time(app: Weak<AppWindow>, ctx: Rc<dyn Context>) {
        ipc::TimestampClient(ctx).get_timestamp_nanos(Box::new(move |t| {
            let t = OffsetDateTime::from_unix_timestamp_nanos(t)
                .unwrap()
                .to_offset(UtcOffset::from_hms(8, 0, 0).unwrap());
            if let Some(ui) = app.upgrade() {
                let home_app = ui.global::<HomeViewModel>();
                home_app.set_time(TimeData {
                    day: t.day() as _,
                    hour: t.hour() as _,
                    minute: t.minute() as _,
                    month: t.month() as _,
                    second: t.second() as _,
                    week: t.weekday().number_days_from_sunday() as _,
                    year: t.year(),
                })
            }
        }))
    }

    fn on_show(&mut self, ctx: Rc<dyn Context>) {
        let app = self.app.clone();
        Self::update_time(app.clone(), ctx.clone());

        let ctx_ref = ctx.clone();
        self.time_update_timer
            .get_or_insert(slint::Timer::default())
            .start(
                slint::TimerMode::Repeated,
                Duration::from_secs(1),
                move || {
                    Self::update_time(app.clone(), ctx_ref.clone());
                },
            );
    }

    fn on_hide(&mut self) {
        self.time_update_timer.take();
    }
}

impl Node for HomePage {
    fn node_name(&self) -> NodeName {
        NodeName::HomePage
    }

    fn handle_message(
        &mut self,
        ctx: Rc<dyn Context>,
        _from: NodeName,
        _to: MessageTo,
        msg: MessageWithHeader,
    ) -> HandleResult {
        match msg.body {
            Message::Lifecycle(msg) => match msg {
                LifecycleMessage::Show => {
                    self.is_show = true;
                    self.on_show(ctx);
                    return HandleResult::Finish(Message::Empty);
                }
                LifecycleMessage::Hide => {
                    self.is_show = false;
                    self.on_hide();
                    return HandleResult::Finish(Message::Empty);
                }
                _ => {}
            },
            Message::OneButton(msg) => {
                if self.is_show {
                    match msg {
                        OneButtonMessage::Click => {
                            ctx.send_message(
                                MessageTo::Point(NodeName::Router),
                                Message::Router(RouterMessage::GotoPage(RoutePage::Menu)),
                            );
                            return HandleResult::Finish(Message::Empty);
                        }
                        _ => {}
                    }
                }
            }
            _ => {}
        }
        HandleResult::Discard
    }
}
