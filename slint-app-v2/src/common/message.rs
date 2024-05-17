use crate::ui::PageRouteTable;
use std::time::Duration;

mod home;
pub use home::HomeMessage;
mod http;
pub use http::{HttpMessage, HttpRequestMethod, HttpBody, HttpResponse};

#[derive(Debug, Clone)]
pub enum LifecycleMessage {
    // 调度器首次调度向所有组件发送一个初始化消息
    Init,
    // 当组件可见时
    Show,
    // 当组件不可见时
    Hide,
}

#[derive(Debug, Clone)]
pub enum OneButtonMessage {
    // 单击
    Click,
    // 点击超过一次
    Clicks(usize),
    // 长按
    LongPressHolding(Duration),
    // 长按松手
    LongPressHeld(Duration),
}

#[derive(Debug, Clone)]
pub enum Message {
    // 空消息
    Empty,
    // App生命周期消息
    Lifecycle(LifecycleMessage),
    // 单按键消息
    OneButton(OneButtonMessage),
    // 路由消息
    Router(PageRouteTable),
    // 首页相关消息
    HomePage(HomeMessage),
    // 天气页相关消息
    WeatherPage,
    // Http消息
    Http(HttpMessage),
}
