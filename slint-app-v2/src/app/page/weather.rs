use crate::common::{Node, NodeName, Context, HandleResult, Message, MessageTo};

pub struct WeatherPage {}

impl WeatherPage {
    pub fn new() -> Self {
        Self {}
    }
}

impl Node for WeatherPage {
    fn node_name(&self) -> NodeName {
        NodeName::WeatherPage
    }

    fn handle_message(
        &mut self,
        _ctx: Box<dyn Context>,
        _from: NodeName,
        _to: MessageTo,
        _msg: Message,
    ) -> HandleResult {
        HandleResult::Discard
    }
}
