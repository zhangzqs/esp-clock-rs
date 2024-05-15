use crate::common::{App, AppName, Context, HandleResult, Message, MessageTo};

pub struct WeatherPageApp {}

impl WeatherPageApp {
    pub fn new() -> Self {
        Self {}
    }
}

impl App for WeatherPageApp {
    fn app_name(&self) -> AppName {
        AppName::WeatherPage
    }

    fn handle_message(
        &mut self,
        _ctx: Box<dyn Context>,
        _from: AppName,
        _to: MessageTo,
        _msg: Message,
    ) -> HandleResult {
        HandleResult::Discard
    }
}
