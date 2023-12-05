use esp_idf_hal::ledc::LedcDriver;

use log::info;
use slint_app::LEDController;
pub struct EspLEDController<'a> {
    ledc: LedcDriver<'a>,
}

impl<'a> EspLEDController<'a> {
    pub fn new(ledc: LedcDriver<'a>) -> Self {
        Self {
            ledc,
        }
    }
}

impl LEDController for EspLEDController<'_> {
    fn get_max_brightness(&self) -> u32 {
        self.ledc.get_max_duty()
    }

    fn set_brightness(&mut self, brightness: u32) {
        info!("set brightness: {}", brightness);
        self.ledc.set_duty(brightness).unwrap();
    }

    fn get_brightness(&self) -> u32 {
        self.ledc.get_duty()
    }
}