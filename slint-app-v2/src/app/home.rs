use std::time::{self, Duration};

use ::time::{OffsetDateTime, UtcOffset};
use slint::{ComponentHandle, Weak};

use super::{AppWindow, AppWindowViewModel};
use crate::{
    common::{App, AppName, Context, Message, MessageTo, SchedulerMessage, Topic},
    scheduler::Scheduler,
};

pub struct HomeApp {
    app: Weak<AppWindow>,
    period_timer: Option<slint::Timer>,
}

impl HomeApp {
    pub fn new(app: Weak<AppWindow>) -> Self {
        Self {
            app,
            period_timer: None,
        }
    }
}

impl App for HomeApp {
    fn app_name(&self) -> AppName {
        AppName::Home
    }

    fn handle_message(
        &mut self,
        ctx: Box<dyn Context>,
        from: AppName,
        to: MessageTo,
        msg: Message,
    ) {
        println!("msg: {:?}", msg);
        match msg {
            Message::Empty => match to {
                MessageTo::Topic(t) => match t {
                    Topic::SecondPeriod => {
                        if let Some(app) = self.app.upgrade() {
                            let a = app.global::<AppWindowViewModel>();
                            let t = OffsetDateTime::now_utc()
                                .to_offset(UtcOffset::from_hms(8, 0, 0).unwrap());
                            a.set_now(format!("{}:{}:{}", t.hour(), t.minute(), t.second()).into());
                        }
                    }
                },
                _ => {}
            },
            Message::SchedulerMessage(msg) => match msg {
                SchedulerMessage::Start => {
                    ctx.subscribe_topic_message(Topic::SecondPeriod);

                    self.period_timer
                        .get_or_insert(slint::Timer::default())
                        .start(
                            slint::TimerMode::Repeated,
                            Duration::from_secs(1),
                            move || {
                                ctx.send_message(
                                    MessageTo::Topic(Topic::SecondPeriod),
                                    Message::Empty,
                                )
                            },
                        );
                }
            },
            _ => {}
        }
    }
}
