use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex};
use std::thread;

use app_core::proto::*;
use esp_idf_hal::rmt::TxRmtDriver;
use esp_idf_hal::rmt::*;
use log::info;
pub struct BuzzerService<'a> {
    tx: Arc<Mutex<TxRmtDriver<'a>>>,
    resp_ready: Arc<Mutex<HashMap<usize, bool>>>,
    playing_flag: Arc<AtomicBool>,
    join_handle: RefCell<Option<thread::JoinHandle<()>>>,
}

impl<'a> BuzzerService<'a> {
    pub fn new(tx: TxRmtDriver<'a>) -> Self {
        Self {
            tx: Arc::new(Mutex::new(tx)),
            resp_ready: Arc::new(Mutex::new(HashMap::new())),
            playing_flag: Arc::new(AtomicBool::new(false)),
            join_handle: RefCell::new(None),
        }
    }

    fn tone(tx: &mut TxRmtDriver<'a>, freq: u16) {
        // 先关闭之前的音
        tx.stop().unwrap();

        if freq != 0 {
            // Calculate the frequency for a piezo buzzer.
            let ticks_hz = tx.counter_clock().unwrap();
            let tick_count = (ticks_hz.0 as u128 / freq as u128 / 2_u128) as u16;
            let ticks = PulseTicks::new(tick_count).unwrap();

            // Add high and low pulses for the tick duration.
            let on = Pulse::new(PinState::High, ticks);
            let off = Pulse::new(PinState::Low, ticks);
            let mut signal = FixedLengthSignal::<1>::new();
            signal.set(0, &(on, off)).unwrap();

            tx.start(signal).unwrap();
        }
    }
}

impl BuzzerService<'_> {
    fn off(&self) {
        if self.playing_flag.load(Ordering::SeqCst) {
            self.playing_flag.store(false, Ordering::SeqCst);
            if let Some(x) = self.join_handle.borrow_mut().take() {
                x.join().unwrap();
            }
        }
        Self::tone(&mut self.tx.lock().unwrap(), 0);
    }
}

impl<'a: 'static> Node for BuzzerService<'a> {
    fn node_name(&self) -> NodeName {
        NodeName::Buzzer
    }

    fn poll(&self, ctx: Rc<dyn Context>, seq: usize) {
        if let Some(x) = self.resp_ready.lock().unwrap().remove(&seq) {
            ctx.async_ready(seq, Message::Buzzer(BuzzerMessage::ToneSeriesResponse(x)));
        }
    }

    fn handle_message(&self, _ctx: Rc<dyn Context>, msg: MessageWithHeader) -> HandleResult {
        let seq = msg.seq;
        if let Message::Buzzer(msg) = msg.body {
            match msg {
                BuzzerMessage::ToneForever(freq) => {
                    self.off();
                    Self::tone(&mut self.tx.lock().unwrap(), freq);
                    return HandleResult::Finish(Message::Empty);
                }
                BuzzerMessage::ToneSeriesRequest(tones) => {
                    info!("receive tone series len: {}", tones.0.len());
                    self.off();

                    let tx = self.tx.clone();
                    let play_flag = self.playing_flag.clone();
                    let resp_ready = self.resp_ready.clone();
                    *self.join_handle.borrow_mut() = Some(thread::spawn(move || {
                        play_flag.store(true, Ordering::SeqCst);
                        for (freq, dur) in tones.0.into_iter() {
                            if play_flag.load(Ordering::SeqCst) {
                                Self::tone(&mut tx.lock().unwrap(), freq);
                                thread::sleep(dur.into());
                            } else {
                                // 收到了一个关闭信号
                                resp_ready.lock().unwrap().insert(seq, false);
                                return;
                            }
                        }
                        resp_ready.lock().unwrap().insert(seq, true);
                    }));
                    return HandleResult::Pending;
                }
                BuzzerMessage::Off => self.off(),
                _ => {}
            }
        }
        HandleResult::Discard
    }
}
