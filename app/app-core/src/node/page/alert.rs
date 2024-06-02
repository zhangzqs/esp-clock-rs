use std::cell::RefCell;
use std::{rc::Rc, time::Duration};

use log::info;
use proto::TopicName;
use slint::{ComponentHandle, Image, SharedPixelBuffer};
use time::OffsetDateTime;

use crate::proto::*;
use crate::storage::WiFiStorage;
use crate::{get_app_window, ui};

pub struct AlertDialog {}

impl AlertDialog {
    pub fn new() -> Self {
        Self {}
    }

    fn show(ctx: Rc<dyn Context>, content: AlertDialogContent) {
        ctx.subscribe_topic(TopicName::OneButton);
        if let Some(ui) = get_app_window().upgrade() {
            let ad = ui.global::<ui::AlertDialogViewModel>();
            ad.set_show(true);
            if let Some(x) = content.text {
                ad.set_text(x.into());
            }
            if let Some(Bytes(_)) = content.image {
                unimplemented!("unsupported");
            }
        }
    }

    fn close(ctx: Rc<dyn Context>) {
        ctx.unsubscribe_topic(TopicName::OneButton);
        if let Some(ui) = get_app_window().upgrade() {
            let ad = ui.global::<ui::AlertDialogViewModel>();
            ad.set_show(false);
            ad.set_text("".into()); // 释放空间
        }
    }

    fn is_show() -> bool {
        if let Some(ui) = get_app_window().upgrade() {
            let ad = ui.global::<ui::AlertDialogViewModel>();
            return ad.get_show();
        }
        panic!("appwindow upgrade failed");
    }
}

impl Node for AlertDialog {
    fn node_name(&self) -> NodeName {
        NodeName::AlertDialog
    }

    fn poll(&self, ctx: Rc<dyn Context>, seq: usize) {
        if !Self::is_show() {
            ctx.async_ready(seq, Message::Empty);
        }
    }

    fn handle_message(&self, ctx: Rc<dyn Context>, msg: MessageWithHeader) -> HandleResult {
        match msg.body {
            Message::AlertDialog(msg) => match msg {
                AlertDialogMessage::ShowRequest { duration, content } => {
                    Self::show(ctx.clone(), content);
                    if let Some(x) = duration {
                        slint::Timer::single_shot(Duration::from_millis(x as _), move || {
                            Self::close(ctx.clone());
                        });
                    }
                    return HandleResult::Pending;
                }
                AlertDialogMessage::Close => {
                    Self::close(ctx.clone());
                    return HandleResult::Finish(Message::Empty);
                }
                _ => {}
            },
            Message::OneButton(OneButtonMessage::Click) => {
                Self::close(ctx.clone());
                return HandleResult::Block;
            }
            _ => {}
        }
        HandleResult::Discard
    }
}
