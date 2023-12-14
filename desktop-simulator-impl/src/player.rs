use embedded_tone::RawTonePlayer;

pub struct RodioPlayer;

impl RodioPlayer {
    pub fn new() -> Self {
        Self
    }
}

unsafe impl Send for RodioPlayer {}


impl RawTonePlayer for RodioPlayer {
    fn tone(&mut self, freq: u32) {
        beep::beep(freq as u16).unwrap();
    }

    fn off(&mut self) {
        beep::beep(0).unwrap();
    }
}
