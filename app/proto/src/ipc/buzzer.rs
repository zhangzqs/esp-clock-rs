use std::rc::Rc;

use crate::{Context, Message, NodeName};

use crate::message::{BuzzerMessage, ToneFrequency, ToneSeries};

use super::AsyncCallback;

#[derive(Clone)]
pub struct BuzzerClient(pub Rc<dyn Context>);

impl BuzzerClient {
    pub fn tone(&self, freq: ToneFrequency) {
        self.0.sync_call(
            NodeName::Buzzer,
            Message::Buzzer(BuzzerMessage::ToneForever(freq)),
        );
    }

    pub fn tone_series(&self, series: ToneSeries, callback: AsyncCallback<bool>) {
        self.0.async_call(
            NodeName::Buzzer,
            Message::Buzzer(BuzzerMessage::ToneSeriesRequest(series)),
            Box::new(|r| {
                callback(match r.unwrap() {
                    Message::Buzzer(BuzzerMessage::ToneSeriesResponse(is_finished)) => is_finished,
                    m => panic!("unexpected response, {:?}", m),
                })
            }),
        );
    }

    pub fn off(&self) {
        self.0
            .sync_call(NodeName::Buzzer, Message::Buzzer(BuzzerMessage::Off));
    }
}
