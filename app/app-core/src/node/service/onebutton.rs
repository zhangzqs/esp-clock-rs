use std::cell::RefCell;
use std::rc::Rc;
use std::time::Duration;

use button_driver::{Button, ButtonConfig, PinWrapper, Platform};

use slint::{ComponentHandle, Weak};

use crate::proto::{
    Context, HandleResult, LifecycleMessage, Message, MessageTo, MessageWithHeader, Node, NodeName,
    OneButtonMessage,
};
use crate::ui::{AppWindow, OneButtenAdapter};

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
    app: Weak<AppWindow>,
    button_event_timer: Option<slint::Timer>,
    button_state: Rc<RefCell<bool>>,
}

impl TouchOneButtonAdapterService {
    pub fn new(app: Weak<AppWindow>) -> Self {
        Self {
            app,
            button_event_timer: None,
            button_state: Rc::new(RefCell::new(false)),
        }
    }
}

impl Node for TouchOneButtonAdapterService {
    fn node_name(&self) -> NodeName {
        NodeName::OneButton
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
                LifecycleMessage::Init => {
                    let mut button = Button::new_with_platform(
                        MyButtonPin(self.button_state.clone()),
                        MyButtonPlatform::new(),
                        ButtonConfig {
                            mode: button_driver::Mode::PullDown, // 当按键松开时，是低电平
                            ..Default::default()
                        },
                    );
                    self.button_event_timer
                        .get_or_insert(slint::Timer::default())
                        .start(
                            slint::TimerMode::Repeated,
                            Duration::from_millis(20),
                            move || {
                                button.tick();
                                if button.clicks() > 0 {
                                    let clicks = button.clicks();
                                    println!("Clicks: {}", clicks);
                                    if clicks == 1 {
                                        ctx.send_message(
                                            MessageTo::Broadcast,
                                            Message::OneButton(OneButtonMessage::Click),
                                        );
                                    } else {
                                        ctx.send_message(
                                            MessageTo::Broadcast,
                                            Message::OneButton(OneButtonMessage::Clicks(clicks)),
                                        );
                                    }
                                } else if let Some(dur) = button.current_holding_time() {
                                    println!("Held for {dur:?}");
                                    ctx.send_message(
                                        MessageTo::Broadcast,
                                        Message::OneButton(OneButtonMessage::LongPressHolding(dur)),
                                    );
                                } else if let Some(dur) = button.held_time() {
                                    println!("Total holding time {dur:?}");
                                    ctx.send_message(
                                        MessageTo::Broadcast,
                                        Message::OneButton(OneButtonMessage::LongPressHeld(dur)),
                                    );
                                }
                                button.reset();
                            },
                        );
                    if let Some(ui) = self.app.upgrade() {
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
