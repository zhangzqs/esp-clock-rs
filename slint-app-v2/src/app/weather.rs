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

    fn handle_message(&self, ctx: Box<dyn Context>, from: AppName, to: MessageTo, msg: Message) {
        match from {
            AppName::Home => {
                ctx.send_message(MessageTo::App(AppName::Home), Message::WeatherMessage);
            }
            _ => {}
        }
    }
}
