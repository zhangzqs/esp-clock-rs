use std::{
    cell::RefCell,
    rc::Rc,
    sync::{Arc, Mutex},
};

use slint::Weak;
use slint_app::{AppWindow, LEDController};

pub struct ScreenBrightnessController {
    app: Arc<Mutex<Weak<AppWindow>>>,
}

impl ScreenBrightnessController {
    pub fn new(app: Arc<Mutex<Weak<AppWindow>>>) -> Self {
        Self { app }
    }
}

impl LEDController for ScreenBrightnessController {
    fn get_max_brightness(&self) -> u32 {
        10000
    }

    fn set_brightness(&mut self, brightness: u32) {
        if let Some(ui) = self.app.lock().unwrap().upgrade() {
            ui.set_mock_brightness(brightness as f32 / self.get_max_brightness() as f32);
        }
    }

    fn get_brightness(&self) -> u32 {
        if let Some(ui) = self.app.lock().unwrap().upgrade() {
            return (ui.get_mock_brightness() * self.get_max_brightness() as f32) as u32;
        }

        0
    }
}
