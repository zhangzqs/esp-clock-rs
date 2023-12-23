use embedded_tone::RawTonePlayer;
use esp_idf_hal::rmt::TxRmtDriver;
use esp_idf_hal::rmt::*;

pub struct EspBeepPlayer<'a> {
    tx: TxRmtDriver<'a>,
}

impl<'a> EspBeepPlayer<'a> {
    pub fn new(tx: TxRmtDriver<'a>) -> Self {
        Self { tx }
    }
}

impl RawTonePlayer for EspBeepPlayer<'_> {
    fn tone(&mut self, freq: u32) {
        // 先关闭之前的音
        self.tx.stop().unwrap();

        // Calculate the frequency for a piezo buzzer.
        let ticks_hz = self.tx.counter_clock().unwrap();
        let tick_count = (ticks_hz.0 as u128 / freq as u128 / 2_u128) as u16;
        let ticks = PulseTicks::new(tick_count).unwrap();

        // Add high and low pulses for the tick duration.
        let on = Pulse::new(PinState::High, ticks);
        let off = Pulse::new(PinState::Low, ticks);
        let mut signal = FixedLengthSignal::<1>::new();
        signal.set(0, &(on, off)).unwrap();

        self.tx.start(signal).unwrap();
    }

    fn off(&mut self) {
        self.tx.stop().unwrap();
    }
}
