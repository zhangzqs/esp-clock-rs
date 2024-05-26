use std::{cell::RefCell, rc::Rc, time::Duration};

use app_core::proto::*;
use button_driver::Button;
use esp_idf_hal::gpio::{Input, Pin, PinDriver};
use esp_idf_sys as _;
use log::info;

pub struct OneButtonService<'a, P: Pin> {
    button: Rc<RefCell<Button<PinDriver<'a, P, Input>, button_driver::DefaultPlatform>>>,
    timer: slint::Timer,
}

impl<'a, P: Pin> OneButtonService<'a, P> {
    pub fn new(pin: PinDriver<'a, P, Input>) -> Self {
        let button = button_driver::Button::new(pin, Default::default());
        Self {
            button: Rc::new(RefCell::new(button)),
            timer: slint::Timer::default(),
        }
    }
}

impl<'a: 'static, P: Pin> Node for OneButtonService<'a, P> {
    fn node_name(&self) -> NodeName {
        NodeName::Other("EspOneButton".into())
    }

    fn handle_message(&self, ctx: Rc<dyn Context>, msg: MessageWithHeader) -> HandleResult {
        if let Message::Lifecycle(LifecycleMessage::Init) = msg.body {
            let button = self.button.clone();
            self.timer.start(
                slint::TimerMode::Repeated,
                Duration::from_millis(20),
                move || {
                    let mut button = button.borrow_mut();
                    button.tick();

                    if button.clicks() > 0 {
                        let clicks = button.clicks();
                        if clicks == 1 {
                            ctx.boardcast(Message::OneButton(OneButtonMessage::Click));
                        } else {
                            ctx.boardcast(Message::OneButton(OneButtonMessage::Clicks(clicks)));
                        }
                    } else if let Some(dur) = button.current_holding_time() {
                        info!("Held for {dur:?}");
                        ctx.boardcast(Message::OneButton(OneButtonMessage::LongPressHolding(dur)));
                    } else if let Some(dur) = button.held_time() {
                        info!("Total holding time {dur:?}");
                        ctx.boardcast(Message::OneButton(OneButtonMessage::LongPressHeld(dur)));
                    }
                    button.reset();
                },
            );
        }
        HandleResult::Discard
    }
}
