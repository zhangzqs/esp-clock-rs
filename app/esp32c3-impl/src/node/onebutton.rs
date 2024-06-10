use std::{
    rc::Rc,
    sync::mpsc::{channel, Receiver, TryRecvError},
    thread,
    time::Duration,
};

use app_core::proto::*;
use esp_idf_hal::gpio::{Input, Pin, PinDriver};
use esp_idf_sys as _;
use log::info;

pub struct OneButtonService {
    rx: Receiver<OneButtonMessage>,
}

impl OneButtonService {
    pub fn new<P: Pin>(pin: PinDriver<'static, P, Input>) -> Self {
        let (tx, rx) = channel();
        let mut button = button_driver::Button::new(pin, Default::default());

        thread::spawn(move || loop {
            button.tick();
            if button.clicks() > 0 {
                let clicks = button.clicks();
                if clicks == 1 {
                    tx.send(OneButtonMessage::Click).unwrap();
                } else {
                    tx.send(OneButtonMessage::Clicks(clicks)).unwrap();
                }
            } else if let Some(dur) = button.current_holding_time() {
                info!("Held for {dur:?}");
                tx.send(OneButtonMessage::LongPressHolding(dur.as_millis() as _))
                    .unwrap();
            } else if let Some(dur) = button.held_time() {
                info!("Total holding time {dur:?}");
                tx.send(OneButtonMessage::LongPressHeld(dur.as_millis() as _))
                    .unwrap();
            }
            button.reset();
            thread::sleep(Duration::from_millis(10));
        });
        Self { rx }
    }
}

impl Node for OneButtonService {
    fn node_name(&self) -> NodeName {
        NodeName::Other("EspOneButton".into())
    }

    fn handle_message(&self, ctx: Rc<dyn Context>, msg: MessageWithHeader) -> HandleResult {
        match msg.body {
            Message::Lifecycle(LifecycleMessage::Init) => {
                ctx.subscribe_topic(TopicName::Scheduler);
            }
            Message::Empty => match self.rx.try_recv() {
                Ok(x) => ctx.broadcast_topic(TopicName::OneButton, Message::OneButton(x)),
                Err(TryRecvError::Empty) => {}
                Err(TryRecvError::Disconnected) => unreachable!(),
            },
            _ => {}
        }
        HandleResult::Discard
    }
}
