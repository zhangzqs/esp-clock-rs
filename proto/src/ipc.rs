use std::rc::Rc;

use crate::{Context, DateTimeMessage, HandleResult, Message, MessageTo, NodeName, UtcDateTime};

pub fn call_datetime_utc_datetime(ctx: Rc<dyn Context>, callback: Box<dyn FnOnce(UtcDateTime)>) {
    ctx.send_message_with_reply_once(
        MessageTo::Point(NodeName::DateTimeClient),
        Message::DateTime(DateTimeMessage::UtcDateTimeRequest),
        Box::new(|_, r| {
            if let HandleResult::Successful(Message::DateTime(
                DateTimeMessage::UtcDateTimeResponse(resp),
            )) = r
            {
                callback(resp);
                return;
            }
            panic!("unexpected response, {:?}", r);
        }),
    )
}
