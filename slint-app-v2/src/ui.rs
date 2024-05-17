use std::{
    sync::{Once},
};

use slint::Weak;
use slint::{ComponentHandle};

slint::include_modules!();

static mut APP: Option<AppWindow> = None;
static APP_ONCE: Once = Once::new();

pub fn get_app_window() -> Weak<AppWindow> {
    APP_ONCE.call_once(|| {
        let app = AppWindow::new().unwrap();
        unsafe { APP = Some(app) }
    });
    unsafe { APP.as_ref().unwrap().as_weak() }
}
