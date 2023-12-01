
pub trait LEDController {
    fn get_max_brightness(&self) -> u32;
    fn set_brightness(&mut self, brightness: u32);
    fn set_brightness_percent(&mut self, brightness_percent: f32) {
        let max_brightness = self.get_max_brightness();
        let brightness = max_brightness as f32 * brightness_percent;
        self.set_brightness(brightness as u32);
    }
}