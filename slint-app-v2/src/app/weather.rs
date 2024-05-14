use crate::common::{App, AppName, Context, Message, MessageTo};

pub struct WeatherApp {}

impl WeatherApp {
    pub fn new() -> Self {
        Self {}
    }
}

impl App for WeatherApp {
    fn app_name(&self) -> AppName {
        AppName::Weather
    }

    fn handle_message(&mut self, ctx: Box<dyn Context>, from: AppName, to: MessageTo, msg: Message) {}
}
