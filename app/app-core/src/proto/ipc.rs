use std::rc::Rc;

use crate::proto::{Context, HandleResult, Message, MessageTo, NodeName, TimeMessage};

use super::{StorageError, StorageMessage};

type Callback<T> = Box<dyn FnOnce(T)>;
type ResultCallback<T, E> = Box<dyn FnOnce(Result<T, E>)>;

pub struct TimestampClient(pub Rc<dyn Context>);

impl TimestampClient {
    pub fn get_timestamp_nanos(&self, callback: Callback<i128>) {
        self.0.send_message_with_reply_once(
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
}

pub struct StorageClient(pub Rc<dyn Context>);

impl StorageClient {
    pub fn set_storage(
        &self,
        key: String,
        value: Option<String>,
        callback: ResultCallback<(), StorageError>,
    ) {
        self.0.send_message_with_reply_once(
            MessageTo::Point(NodeName::Storage),
            Message::Storage(StorageMessage::SetRequest(key, value)),
            Box::new(|_, r| {
                callback(r.map(
                    |_| (),
                    |e| {
                        if let Message::Storage(StorageMessage::Error(err)) = e {
                            return err;
                        }
                        panic!("not err");
                    },
                ))
            }),
        )
    }
    pub fn get_storage(&self, key: String, callback: ResultCallback<Option<String>, StorageError>) {
        self.0.send_message_with_reply_once(
            MessageTo::Point(NodeName::Storage),
            Message::Storage(StorageMessage::GetRequest(key)),
            Box::new(|_, r| {}),
        );
    }

    pub fn list_keys(&self, callback: ResultCallback<(), StorageError>) {
        self.0.send_message_with_reply_once(
            MessageTo::Point(NodeName::Storage),
            Message::Storage(StorageMessage::ListKeysRequest),
            Box::new(|_, r| {}),
        );
    }
}
