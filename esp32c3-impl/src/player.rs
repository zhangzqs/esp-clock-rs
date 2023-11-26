use embedded_tone::{NoteDuration, Player};
use esp_idf_hal::rmt::TxRmtDriver;
use std::time::Duration;
use std::thread;

pub struct EspBeepPlayer<'a> {
    beat_duration: std::time::Duration,
    tx: TxRmtDriver<'a>,
}

impl<'a> EspBeepPlayer<'a> {
    pub fn new(tx: TxRmtDriver<'a>) -> Self {
        let mut ret = Self {
            beat_duration: Duration::ZERO,
            tx,
        };
        ret.set_beat_duration_from_bpm(60, NoteDuration::Quarter);
        ret
    }
}

impl<'a> Player for EspBeepPlayer<'a> {
    fn play_note(&mut self, note: embedded_tone::Note) {
        use esp_idf_hal::rmt::*;
        let pitch = note.pitch.frequency();
        let duration = self.beat_duration.mul_f32(note.duration.into());
        // Calculate the frequency for a piezo buzzer.
        let ticks_hz = self.tx.counter_clock().unwrap();
        let tick_count = (ticks_hz.0 as u128 / pitch as u128 / 2_u128) as u16;
        let ticks = PulseTicks::new(tick_count).unwrap();

        // Add high and low pulses for the tick duration.
        let on = Pulse::new(PinState::High, ticks);
        let off = Pulse::new(PinState::Low, ticks);
        let mut signal = FixedLengthSignal::<1>::new();
        signal.set(0, &(on, off)).unwrap();

        self.tx.start(signal).unwrap();
        thread::sleep(duration);
        self.tx.stop().unwrap();
    }

    fn play_rest(&mut self, rest: embedded_tone::Rest) {
        thread::sleep(self.beat_duration.mul_f32(rest.duration.into()));
    }

    fn set_beat_duration(&mut self, beat_duration: Duration) {
        self.beat_duration = beat_duration;
    }
}
