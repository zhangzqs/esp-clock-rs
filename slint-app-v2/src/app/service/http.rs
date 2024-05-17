use slint::{ComponentHandle, Weak};

use crate::common::{App, AppName, Context, HandleResult, LifecycleMessage, Message, MessageTo};
use crate::ui::{AppWindow, PageRouteTable, PageRouter};

pub struct HttpClientApp {}

impl HttpClientApp {
    pub fn new() -> Self {
        Self {}
    }
}

impl App for HttpClientApp {
    fn app_name(&self) -> AppName {
        AppName::HttpClient
    }

    fn handle_message(
        &mut self,
        ctx: Box<dyn Context>,
        _from: AppName,
        _to: MessageTo,
        msg: Message,
    ) -> HandleResult {
        match msg {
            
            _ => {}
        }
        HandleResult::Discard
    }
}
