use slint_app::LEDController;

pub struct MockLEDController {
    brightness: u32,
    max_brightness: u32,
}

impl MockLEDController {
    pub fn new() -> Self {
        Self {
            brightness: 0,
            max_brightness: 100,
        }
    }
}

impl LEDController for MockLEDController {
    fn get_max_brightness(&self) -> u32 {
        self.max_brightness
    }

    fn set_brightness(&mut self, brightness: u32) {
        self.brightness = brightness;
    }

    fn get_brightness(&self) -> u32 {
        self.brightness
    }
}
