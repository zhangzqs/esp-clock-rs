use std::rc::Rc;

use slint::{ComponentHandle, Model, Weak};

use crate::common::{
    App, AppName, Context, HandleResult, LifecycleMessage, Message, MessageTo, OneButtonMessage,
};
use crate::ui::{AppWindow, MenuViewModel};

pub struct MenuPageApp {
    app: Weak<AppWindow>,
    is_show: bool,
}

impl MenuPageApp {
    pub fn new(app: Weak<AppWindow>) -> Self {
        Self {
            app,
            is_show: false,
        }
    }

    fn next_page(&self) {
        if let Some(ui) = self.app.upgrade() {
            let menu = ui.global::<MenuViewModel>();
            let total_size = menu.get_entry_list().row_count();
            menu.set_current_id((menu.get_current_id() + 1) % total_size as i32);
        }
    }

    fn enter_page(&self, ctx: Rc<Box<dyn Context>>) {
        if let Some(ui) = self.app.upgrade() {
            let menu = ui.global::<MenuViewModel>();
            if let Some(x) = menu
                .get_entry_list()
                .row_data(menu.get_current_id() as usize)
            {
                ctx.send_message(MessageTo::App(AppName::Router), Message::Router(x.page))
            }
        }
    }
}

impl App for MenuPageApp {
    fn app_name(&self) -> AppName {
        AppName::MenuPage
    }

    fn handle_message(
        &mut self,
        ctx: Box<dyn Context>,
        _from: AppName,
        _to: MessageTo,
        msg: Message,
    ) -> HandleResult {
        match msg {
            Message::Lifecycle(msg) => match msg {
                LifecycleMessage::Show => self.is_show = true,
                LifecycleMessage::Hide => self.is_show = false,
                _ => {}
            },
            Message::OneButton(msg) => {
                if self.is_show {
                    match msg {
                        OneButtonMessage::Click => self.next_page(),
                        OneButtonMessage::Clicks(2) => self.enter_page(Rc::new(ctx)),
                        _ => {}
                    }
                }
            }
            _ => {}
        }
        HandleResult::Discard
    }
}
