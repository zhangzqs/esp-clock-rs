mod time;
pub use time::{DateTimeMessage, UtcDateTime};

mod http;
pub use http::{HttpBody, HttpMessage, HttpRequest, HttpRequestMethod, HttpResponse};

mod router;
pub use router::{RoutePage, RouterMessage};

mod onebutton;
pub use onebutton::OneButtonMessage;

mod lifecycle;
pub use lifecycle::LifecycleMessage;

#[derive(Debug, Clone)]
pub enum Message {
    // 空消息
    Empty,
    // App生命周期消息
    Lifecycle(LifecycleMessage),
    // 单按键消息
    OneButton(OneButtonMessage),
    // 路由消息
    Router(RouterMessage),
    // 天气页相关消息
    WeatherPage,
    // Http消息
    Http(HttpMessage),
    // 时间日期相关消息
    DateTime(DateTimeMessage),
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
                },
            },
            Message::WeatherPage => "weather",
            Message::Http(msg) => match msg {
                HttpMessage::Request(_) => "http/request",
                HttpMessage::Response(_) => "http/response",
            },
            Message::DateTime(_) => "datetime/*",
        }
    }
}
