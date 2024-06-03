mod time;
use std::time::Duration;

use serde::{Deserialize, Serialize};
pub use time::TimeMessage;

mod http;
pub use http::*;

mod router;
pub use router::{RoutePage, RouterMessage};

mod onebutton;
pub use onebutton::OneButtonMessage;

mod lifecycle;
pub use lifecycle::LifecycleMessage;

mod weather;
pub use weather::*;

mod storage;
pub use storage::*;

mod performance;
pub use performance::*;

mod wifi;
pub use wifi::*;

mod buzzer;
pub use buzzer::*;

mod midi;
pub use midi::*;

mod common;
pub use common::*;

mod alertdialog;
pub use alertdialog::*;

mod alarm;
pub use alarm::*;

mod useralarm;
pub use useralarm::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Message {
    /// 空消息
    Empty,
    /// App生命周期消息
    Lifecycle(LifecycleMessage),
    /// 单按键消息
    OneButton(OneButtonMessage),
    /// 路由消息
    Router(RouterMessage),
    /// 天气页相关消息
    Weather(WeatherMessage),
    /// Http消息
    Http(HttpMessage),
    /// 时间日期相关消息
    DateTime(TimeMessage),
    /// 本地存储相关消息
    Storage(StorageMessage),
    /// 性能相关消息
    Performance(PerformanceMessage),
    Timer(TimerMessage),
    WiFi(WiFiMessage),
    Buzzer(BuzzerMessage),
    Midi(MidiMessage),
    AlertDialog(AlertDialogMessage),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TimerMessage {
    Request(Duration),
    Response,
}

impl Message {
    pub fn debug_msg(&self) -> &'static str {
        match self {
            Message::Empty => "empty",
            Message::Lifecycle(msg) => match msg {
                LifecycleMessage::Init => "lifecycle/init",
                LifecycleMessage::Show => "lifecycle/show",
                LifecycleMessage::Hide => "lifecycle/hide",
            },
            Message::OneButton(msg) => match msg {
                OneButtonMessage::Click => "onebutton/click",
                OneButtonMessage::Clicks(_) => "onebutton/clicks",
                OneButtonMessage::LongPressHolding(_) => "onebutton/holding",
                OneButtonMessage::LongPressHeld(_) => "onebutton/held",
            },
            Message::Router(msg) => match msg {
                RouterMessage::GotoPage(msg) => match msg {
                    RoutePage::Boot => "router/gotopage/boot",
                    RoutePage::Home => "router/gotopage/home",
                    RoutePage::Menu => "router/gotopage/menu",
                    RoutePage::Weather => "router/gotopage/weather",
                    RoutePage::Music => "router/gotopage/music",
                },
            },
            Message::Weather(_) => "weather",
            Message::Http(msg) => match msg {
                HttpMessage::Error(_) => "http/error",
                HttpMessage::Request(_) => "http/request",
                HttpMessage::Response(_) => "http/response",
            },
            Message::DateTime(_) => "datetime/*",
            Message::Storage(_) => "storage",
            Message::Performance(_) => "performance",
            _ => "unknown",
        }
    }
}

#[test]
fn test_message_size() {
    let s = std::mem::size_of::<Message>();
    println!("Message size {}", s);
}
