use std::rc::Rc;

use crate::{Context, Message, NodeName, NotifactionContent, NotifactionMessage};

use super::AsyncCallback;
#[derive(Clone)]
pub struct NotifactionClient(pub Rc<dyn Context>);

impl NotifactionClient {
    pub fn show(&self, duration: usize, content: NotifactionContent, on_close: AsyncCallback<()>) {
        self.0.async_call(
            NodeName::Notifaction,
            Message::Notifaction(NotifactionMessage::ShowRequest { duration, content }),
            Box::new(move |r| match r.unwrap() {
                Message::Notifaction(NotifactionMessage::ShowResponse) => {
                    on_close(());
                }
                m => panic!("unexcepted msg: {:?}", m),
            }),
        )
    }

    pub fn close(&self) {
        self.0.sync_call(
            NodeName::Notifaction,
            Message::Notifaction(NotifactionMessage::Close),
        );
    }
}
