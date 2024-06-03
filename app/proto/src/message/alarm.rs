use crate::TopicName;

type Crontab = String;

pub enum AlarmMessage {
    AddRequest(Crontab, TopicName),
    AddResponse(usize),

    DeleteRequest(usize),
    DeleteResponse(Crontab, TopicName),

    GetRequest(usize),
    GetResponse(Crontab, TopicName),

    ListRequest,
    ListResponse(Vec<usize>),
}
