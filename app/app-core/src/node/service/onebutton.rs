use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use button_driver::{Button, ButtonConfig, PinWrapper, Platform};

use log::info;
use proto::TopicName;
use slint::ComponentHandle;

use crate::get_app_window;
use crate::proto::{
    Context, HandleResult, LifecycleMessage, Message, MessageWithHeader, Node, NodeName,
    OneButtonMessage,
};
use crate::ui::OneButtenAdapter;

#[derive(Clone)]
struct MyButtonPin(Rc<RefCell<bool>>);

impl PinWrapper for MyButtonPin {
    fn is_high(&self) -> bool {
        *self.0.borrow()
    }
}

struct MyButtonPlatform {
    _t: slint::Timer,
    dur: Rc<RefCell<Duration>>,
}

impl MyButtonPlatform {
    fn new() -> Self {
        let dur = Rc::new(RefCell::new(Duration::ZERO));
        let t = slint::Timer::default();
        let dur1 = dur.clone();
        t.start(
            slint::TimerMode::Repeated,
            Duration::from_millis(20),
            move || *dur1.borrow_mut() += Duration::from_millis(20),
        );
        Self { _t: t, dur }
    }
}

impl Platform for MyButtonPlatform {
    fn duration_since_init(&self) -> Duration {
        *self.dur.borrow()
    }
}

// 基于触摸事件模拟的单按钮事件的适配器服务
pub struct TouchOneButtonAdapterService {
    button_event_timer: slint::Timer,
    button_state: Rc<RefCell<bool>>,
}

impl TouchOneButtonAdapterService {
    pub fn new() -> Self {
        Self {
            button_event_timer: slint::Timer::default(),
            button_state: Rc::new(RefCell::new(false)),
        }
    }
}

impl Node for TouchOneButtonAdapterService {
    fn node_name(&self) -> NodeName {
        NodeName::OneButton
    }

    fn handle_message(&self, ctx: Rc<dyn Context>, msg: MessageWithHeader) -> HandleResult {
        match msg.body {
            Message::Lifecycle(msg) => match msg {
                LifecycleMessage::Init => {
                    info!("TouchOneButtonAdapterService init");
                    let mut button = Button::new_with_platform(
                        MyButtonPin(self.button_state.clone()),
                        MyButtonPlatform::new(),
                        ButtonConfig {
                            mode: button_driver::Mode::PullDown, // 当按键松开时，是低电平
                            ..Default::default()
                        },
                    );
                    self.button_event_timer.start(
                        slint::TimerMode::Repeated,
                        Duration::from_millis(20),
                        move || {
                            button.tick();
                            if button.clicks() > 0 {
                                let clicks = button.clicks();
                                info!("Clicks: {}", clicks);
                                if clicks == 1 {
                                    ctx.broadcast_topic(
                                        TopicName::OneButton,
                                        Message::OneButton(OneButtonMessage::Click),
                                    );
                                } else {
                                    ctx.broadcast_topic(
                                        TopicName::OneButton,
                                        Message::OneButton(OneButtonMessage::Clicks(clicks)),
                                    );
                                }
                            } else if let Some(dur) = button.current_holding_time() {
                                info!("Held for {dur:?}");
                                ctx.broadcast_topic(
                                    TopicName::OneButton,
                                    Message::OneButton(OneButtonMessage::LongPressHolding(
                                        dur.as_millis() as _,
                                    )),
                                );
                            } else if let Some(dur) = button.held_time() {
                                info!("Total holding time {dur:?}");
                                ctx.broadcast_topic(
                                    TopicName::OneButton,
                                    Message::OneButton(OneButtonMessage::LongPressHeld(
                                        dur.as_millis() as _,
                                    )),
                                );
                            }
                            button.reset();
                        },
                    );
                    if let Some(ui) = get_app_window().upgrade() {
                        let button_state_ref = self.button_state.clone();
                        let t = ui.global::<OneButtenAdapter>();
                        t.on_pressed(move || {
                            *button_state_ref.borrow_mut() = true;
                        });
                        let button_state_ref = self.button_state.clone();
                        t.on_release(move || {
                            *button_state_ref.borrow_mut() = false;
                        });
                    }
                    return HandleResult::Finish(Message::Empty);
                }
                _ => {}
            },
            _ => {}
        }
        HandleResult::Discard
    }
}
