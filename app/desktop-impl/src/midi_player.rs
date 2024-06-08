use app_core::proto::*;
use std::rc::Rc;
pub struct MidiPlayer {}

impl MidiPlayer {
    pub fn new() -> Self {
        Self {}
    }
}

impl Node for MidiPlayer {
    fn node_name(&self) -> NodeName {
        NodeName::Midi
    }

    fn handle_message(&self, _ctx: Rc<dyn Context>, msg: MessageWithHeader) -> HandleResult {
        match msg.body {
            Message::Midi(MidiMessage::PlayRequest(r)) => {
                println!("{r:?}");
                return HandleResult::Finish(Message::Midi(MidiMessage::PlayResponse(false)));
            }
            _ => {}
        }
        HandleResult::Discard
    }
}
