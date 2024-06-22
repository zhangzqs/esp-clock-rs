use std::rc::Rc;

use crate::{Context, Message, NodeName, UserAlarmBody, UserAlarmError, UserAlarmMessage};

type Result<T> = std::result::Result<T, UserAlarmError>;

#[derive(Clone)]
pub struct UserAlarmClient(pub Rc<dyn Context>);

impl UserAlarmClient {
    pub fn add(&self, body: UserAlarmBody) -> Result<usize> {
        match self
            .0
            .sync_call(
                NodeName::Alarm,
                Message::UserAlarm(UserAlarmMessage::AddRequest(body)),
            )
            .unwrap()
        {
            Message::UserAlarm(msg) => match msg {
                UserAlarmMessage::AddResponse(ret) => Ok(ret),
                UserAlarmMessage::Error(e) => Err(e),
                m => panic!("unexcepted msg: {m:?}"),
            },
            m => panic!("unexcepted msg: {m:?}"),
        }
    }

    pub fn delete(&self, id: usize) -> Result<UserAlarmBody> {
        match self
            .0
            .sync_call(
                NodeName::Alarm,
                Message::UserAlarm(UserAlarmMessage::DeleteRequest(id)),
            )
            .unwrap()
        {
            Message::UserAlarm(msg) => match msg {
                UserAlarmMessage::DeleteResponse(ret) => Ok(ret),
                UserAlarmMessage::Error(e) => Err(e),
                m => panic!("unexcepted msg: {m:?}"),
            },
            m => panic!("unexcepted msg: {m:?}"),
        }
    }
    pub fn get(&self, id: usize) -> Result<UserAlarmBody> {
        match self
            .0
            .sync_call(
                NodeName::Alarm,
                Message::UserAlarm(UserAlarmMessage::GetRequest(id)),
            )
            .unwrap()
        {
            Message::UserAlarm(msg) => match msg {
                UserAlarmMessage::GetResponse(ret) => Ok(ret),
                UserAlarmMessage::Error(e) => Err(e),
                m => panic!("unexcepted msg: {m:?}"),
            },
            m => panic!("unexcepted msg: {m:?}"),
        }
    }
    pub fn list(&self) -> Vec<usize> {
        match self
            .0
            .sync_call(
                NodeName::Alarm,
                Message::UserAlarm(UserAlarmMessage::ListRequest),
            )
            .unwrap()
        {
            Message::UserAlarm(msg) => match msg {
                UserAlarmMessage::ListResponse(ret) => ret,
                m => panic!("unexcepted msg: {m:?}"),
            },
            m => panic!("unexcepted msg: {m:?}"),
        }
    }
}
