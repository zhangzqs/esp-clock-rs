use crate::get_app_window;
use crate::proto::{
    ipc, Context, HandleResult, LifecycleMessage, Message, MessageTo, MessageWithHeader, Node,
    NodeName, OneButtonMessage, RoutePage, RouterMessage,
};
use crate::ui::{AppWindow, HomeViewModel, TimeData};
use slint::{ComponentHandle, Weak};
use std::cell::RefCell;
use std::{rc::Rc, time::Duration};
use time::{OffsetDateTime, UtcOffset};

pub struct HomePage {
    time_update_timer: RefCell<Option<slint::Timer>>,
    is_show: RefCell<bool>,
}

impl HomePage {
    pub fn new() -> Self {
        Self {
            time_update_timer: RefCell::new(None),
            is_show: RefCell::new(false),
        }
    }
}

impl HomePage {
    fn update_time(app: Weak<AppWindow>, ctx: Rc<dyn Context>) {
        let t = ipc::TimestampClient(ctx).get_timestamp_nanos();
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
    }

    fn on_show(&self, ctx: Rc<dyn Context>) {
        let app = get_app_window();
        Self::update_time(app.clone(), ctx.clone());

        let ctx_ref = ctx.clone();
        self.time_update_timer
            .borrow_mut()
            .get_or_insert(slint::Timer::default())
            .start(
                slint::TimerMode::Repeated,
                Duration::from_secs(1),
                move || {
                    Self::update_time(app.clone(), ctx_ref.clone());
                },
            );
    }

    fn on_hide(&self) {
        self.time_update_timer.borrow_mut().take();
    }
}

impl Node for HomePage {
    fn node_name(&self) -> NodeName {
        NodeName::HomePage
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
                LifecycleMessage::Show => {
                    *self.is_show.borrow_mut() = true;
                    self.on_show(ctx);
                    return HandleResult::Finish(Message::Empty);
                }
                LifecycleMessage::Hide => {
                    *self.is_show.borrow_mut() = false;
                    self.on_hide();
                    return HandleResult::Finish(Message::Empty);
                }
                _ => {}
            },
            Message::OneButton(msg) => {
                if *self.is_show.borrow() {
                    match msg {
                        OneButtonMessage::Click => {
                            ctx.sync_call(
                                NodeName::Router,
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
