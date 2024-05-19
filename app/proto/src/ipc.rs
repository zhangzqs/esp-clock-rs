use std::rc::Rc;

use crate::{Context, TimeMessage, HandleResult, Message, MessageTo, NodeName};

pub fn get_timestamp_nanos(ctx: Rc<dyn Context>, callback: Box<dyn FnOnce(i128)>) {
    ctx.send_message_with_reply_once(
        MessageTo::Point(NodeName::TimestampClient),
        Message::DateTime(TimeMessage::GetTimestampNanosRequest),
        Box::new(|_, r| {
            if let HandleResult::Successful(Message::DateTime(
                TimeMessage::GetTimestampNanosResponse(resp),
            )) = r
            {
                callback(resp);
                return;
            }
            panic!("unexpected response, {:?}", r);
        }),
    )
}
