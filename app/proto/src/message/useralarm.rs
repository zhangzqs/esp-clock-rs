use crate::TopicName;

pub struct UserAlarmBody {
    pub crontab: String,
    pub music: Option<String>,

    /// 闹钟到达时广播一个话题
    pub topic: TopicName,
}

pub enum UserAlarmMessage {
    AddRequest(UserAlarmBody),
    AddResponse(usize),

    DeleteRequest(usize),
    DeleteResponse(UserAlarmBody),

    GetRequest(usize),
    GetResponse(UserAlarmBody),

    ListRequest,
    ListResponse(Vec<usize>),
}
