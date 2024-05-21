use std::rc::Rc;

use time::OffsetDateTime;

pub struct TimestampClientService {}

impl proto::Node for TimestampClientService {
    fn node_name(&self) -> proto::NodeName {
        proto::NodeName::TimestampClient
    }

    fn handle_message(
        &mut self,
        _ctx: Rc<dyn proto::Context>,
        _from: proto::NodeName,
        _to: proto::MessageTo,
        msg: proto::MessageWithHeader,
    ) -> proto::HandleResult {
        if let proto::Message::DateTime(proto::TimeMessage::GetTimestampNanosRequest) = msg.body {
            return proto::HandleResult::Successful(proto::Message::DateTime(
                proto::TimeMessage::GetTimestampNanosResponse(
                    OffsetDateTime::now_utc().unix_timestamp_nanos(),
                ),
            ));
        }
        proto::HandleResult::Discard
    }
}
