use std::rc::Rc;

use crate::{Context, Message, NodeName};

use crate::message::{Bytes, MidiError, MidiMessage};

use super::AsyncResultCallback;

#[derive(Clone)]
pub struct MidiPlayerClient(pub Rc<dyn Context>);

impl MidiPlayerClient {
    pub fn play(&self, mid: Vec<u8>, callback: AsyncResultCallback<bool, MidiError>) {
        self.0.async_call(
            NodeName::MidiPlayer,
            Message::Midi(MidiMessage::PlayRequest(Bytes(mid))),
            Box::new(|r| {
                callback(match r.unwrap() {
                    Message::Midi(msg) => match msg {
                        MidiMessage::PlayResponse(is_finished) => Ok(is_finished),
                        MidiMessage::Error(e) => Err(e),
                        m => panic!("unexpected response, {:?}", m),
                    },
                    m => panic!("unexpected response, {:?}", m),
                });
            }),
        );
    }

    pub fn off(&self) {
        self.0
            .sync_call(NodeName::MidiPlayer, Message::Midi(MidiMessage::Off));
    }
}
