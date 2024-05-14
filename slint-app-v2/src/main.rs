use std::{cell::RefCell, collections::HashMap, rc::Rc};

mod app;
mod common;
mod scheduler;
use app::{HomeApp, WeatherApp};
use common::{App, AppName, Context, Message, MessageTo, Topic};
use scheduler::Scheduler;
fn main() {
    let mut sche = Scheduler::new();
    sche.register_app(HomeApp::new());
    sche.register_app(WeatherApp::new());
    loop {
        sche.schedule_once();
    }
}
