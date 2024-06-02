use app_core::proto::*;
use base64::{prelude::BASE64_STANDARD, Engine};

use wasm_bindgen::prelude::*;
#[wasm_bindgen(raw_module = "../index.js")]
extern "C" {
    fn loadFile(b64: String);
}

pub struct MidiPlayerService {}

impl MidiPlayerService {
    pub fn new() -> Self {
        Self {}
    }
}

impl Node for MidiPlayerService {
    fn node_name(&self) -> NodeName {
        NodeName::Midi
    }

    fn handle_message(
        &self,
        _ctx: std::rc::Rc<dyn Context>,
        msg: MessageWithHeader,
    ) -> HandleResult {
        match msg.body {
            Message::Midi(MidiMessage::PlayRequest(Bytes(bs))) => {
                let mut s = String::new();
                BASE64_STANDARD.encode_string(bs, &mut s);
                loadFile(s);
                return HandleResult::Finish(Message::Midi(MidiMessage::PlayResponse(false)));
            }
            _ => {}
        }
        HandleResult::Discard
    }
}
