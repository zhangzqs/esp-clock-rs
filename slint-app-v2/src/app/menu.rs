use crate::common::{App, AppName, Context, Message, MessageTo};

pub struct MenuApp {}

impl App for MenuApp {
    fn app_name(&self) -> crate::common::AppName {
        AppName::MenuPage
    }

    fn handle_message(
        &mut self,
        _ctx: Box<dyn Context>,
        _from: AppName,
        _to: MessageTo,
        _msg: Message,
    ) {
        todo!()
    }
}
