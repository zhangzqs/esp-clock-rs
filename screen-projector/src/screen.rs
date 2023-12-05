use scrap::Display;
use std::io::ErrorKind::WouldBlock;
use std::slice::from_raw_parts;
use std::time::Duration;

pub struct Capturer {
    w: usize,
    h: usize,
    capturer: Option<scrap::Capturer>,
    sleep: Duration,
}
impl Capturer {
    pub fn new() -> Capturer {
        let display = Display::primary().unwrap();
        let capturer = scrap::Capturer::new(display).unwrap();
        let (w, h) = (capturer.width(), capturer.height());
        Capturer {
            w,
            h,
            capturer: Some(capturer),
            sleep: Duration::new(1, 0) / 120,
        }
    }
    fn reload(&mut self) {
        println!("Reload capturer");
        drop(self.capturer.take());
        let display = match Display::primary() {
            Ok(display) => display,
            Err(_) => {
                return;
            }
        };

        let capturer = match scrap::Capturer::new(display) {
            Ok(capturer) => capturer,
            Err(_) => return,
        };
        self.capturer = Some(capturer);
    }
    pub fn size(&self) -> (usize, usize) {
        (self.w, self.h)
    }

    pub fn capture(&mut self) -> &[u8] {
        loop {
            match &mut self.capturer {
                Some(capturer) => {
                    let cp = capturer.frame();
                    let buffer = match cp {
                        Ok(buffer) => buffer,
                        Err(error) => {
                            std::thread::sleep(self.sleep);
                            if error.kind() != WouldBlock {
                                std::thread::sleep(std::time::Duration::from_millis(20));
                                self.reload();
                            }
                            continue;
                        }
                    };
                    return unsafe { from_raw_parts(buffer.as_ptr(), buffer.len()) };
                }
                None => {
                    std::thread::sleep(std::time::Duration::from_millis(20));
                    self.reload();
                    continue;
                }
            };
        }
    }
}
