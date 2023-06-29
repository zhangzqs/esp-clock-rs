use std::ops::Range;
use std::rc::Rc;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use button_driver::{Button, ButtonConfig};
use embedded_graphics::{pixelcolor::Rgb565, prelude::*};
use embedded_hal::spi::MODE_3;
use esp_idf_hal::{gpio::{PinDriver, AnyIOPin}, prelude::*, spi::{SpiDeviceDriver, SpiDriverConfig, config::Config}, delay::Ets};
use esp_idf_sys as _; // If using the `binstart` feature of `esp-idf-sys`, always keep this module imported
use log::*;
use display_interface_spi::{SPIInterface, SPIInterfaceNoCS};
use embedded_graphics::mono_font::iso_8859_16::FONT_10X20;
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::pixelcolor::raw::RawU16;
use embedded_graphics::prelude::DrawTarget;
use embedded_graphics::primitives::{Circle, Line, PrimitiveStyle, PrimitiveStyleBuilder, Rectangle, StrokeAlignment, Triangle};
use embedded_graphics::text::{Alignment, Text};
use esp_idf_hal::gpio::Gpio8;
use esp_idf_hal::spi::SPI1;
use esp_idf_hal::task::watchdog::{TWDTConfig, TWDTDriver};
use esp_idf_svc::{eventloop::EspSystemEventLoop, systime::EspSystemTime};
use mipidsi::{Builder, ColorInversion, ColorOrder, Orientation};

use slint::platform::{Key, Platform, WindowAdapter, WindowEvent};
use slint::platform::software_renderer::{LineBufferProvider, MinimalSoftwareWindow, Rgb565Pixel};
use slint::PlatformError;

mod wifi;

struct MyPlatform {
    window: Rc<MinimalSoftwareWindow>,
}

impl Platform for MyPlatform {
    fn create_window_adapter(&self) -> Result<Rc<dyn WindowAdapter>, PlatformError> {
        Ok(self.window.clone())
    }

    fn duration_since_start(&self) -> Duration {
        EspSystemTime {}.now()
    }
}
struct MyLineBufferProvider <'a, T>
where T: DrawTarget<Color = Rgb565>{
    display: &'a mut T,
}

impl <T: DrawTarget<Color = Rgb565>> LineBufferProvider for MyLineBufferProvider<'_, T> {
    type TargetPixel = Rgb565Pixel;

    fn process_line(
        &mut self,
        line: usize,
        range: Range<usize>,
        render_fn: impl FnOnce(&mut [Self::TargetPixel]),
    ) {
        let mut buf = vec![Rgb565Pixel::default(); range.len()];
        let rect = Rectangle::new(
            Point::new(range.start as _, line as _),
            Size::new(range.len() as _, 1),
        );
        render_fn(&mut buf[range]);
        self.display
            .fill_contiguous(
                &rect,
                buf.iter()
                    .map(|p| Rgb565::from(RawU16::from(p.0)))
            )
            .map_err(drop)
            .unwrap();
    }
}

enum ButtonEvent {
    Clicked,
    DoubleClicked,
    TripleClicked,
    Pressed(Duration),
    Released(Duration),
}

fn main() ->anyhow::Result<()>{
    // It is necessary to call this function once. Otherwise some patches to the runtime
    // implemented by esp-idf-sys might not link properly. See https://github.com/esp-rs/esp-idf-template/issues/71
    esp_idf_sys::link_patches();
    // Bind the log crate to the ESP Logging facilities
    esp_idf_svc::log::EspLogger::initialize_default();
    let sysloop = EspSystemEventLoop::take()?;

    let peripherals = Peripherals::take().unwrap();

    let wdt = peripherals.twdt;
    let mut config = TWDTConfig::default();
    config.duration = Duration::from_secs(100);
    let mut wdt  = TWDTDriver::new(wdt, &config)?;
    info!("main start!!!");

    let mut led = PinDriver::output(peripherals.pins.gpio2)?;
    let btn = PinDriver::input(peripherals.pins.gpio9)?;
    let cs = PinDriver::output(peripherals.pins.gpio5)?;
    let dc = PinDriver::output(peripherals.pins.gpio4)?;
    let mut rst = PinDriver::output(peripherals.pins.gpio8)?;

    let mut delay = Ets;
    let mut spi = SpiDeviceDriver::new_single(
        peripherals.spi2,
        peripherals.pins.gpio6,
        peripherals.pins.gpio7,
        Option::<AnyIOPin>::None,
        Option::<AnyIOPin>::None,
        &SpiDriverConfig::default(),
        &Config::default().baudrate(80.MHz().into()).data_mode(MODE_3),
    )?;
    info!("SPI init done");
    let di = SPIInterface::new(spi, dc, cs);
    let mut display = Builder::st7789(di)
        .with_display_size(240,240)
        .with_framebuffer_size(240,240)
        .with_orientation(Orientation::Portrait(false))
        .with_invert_colors(ColorInversion::Inverted)
        .init(&mut delay, Some(rst))
        .unwrap();
    info!("display init done");

    let mut button = Button::new(btn, ButtonConfig::default());

    thread::spawn(move || {
        loop {
            led.toggle().unwrap();
            thread::sleep(Duration::from_millis(1000));
        }
    });

    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        loop {
            button.tick();
            if button.is_clicked() {
                info!("Click");
                tx.send(ButtonEvent::Clicked).unwrap();
            } else if button.is_double_clicked() {
                info!("Double click");
                tx.send(ButtonEvent::DoubleClicked).unwrap();
            } else if button.is_triple_clicked() {
                info!("Triple click");
                tx.send(ButtonEvent::TripleClicked).unwrap();
            } else if let Some(dur) = button.current_holding_time() {
                info!("Held for {dur:?}");
                tx.send(ButtonEvent::Pressed(dur)).unwrap();
            } else if let Some(dur) = button.held_time() {
                info!("Total holding time {dur:?}");
                tx.send(ButtonEvent::Pressed(dur)).unwrap();
            }
            button.reset();
            thread::sleep(Duration::from_millis(50));
        }
    });



    let window = MinimalSoftwareWindow::new(Default::default());
    slint::platform::set_platform(Box::new(MyPlatform {
        window: window.clone(),
    })).unwrap();
    let ui = slint_app::create_app();
    
    display.clear(Rgb565::GREEN).expect("clear failed");
    loop {
        slint::platform::update_timers_and_animations();
        window.draw_if_needed(|renderer| {
            let provider = MyLineBufferProvider {
                display: &mut display,
            };
            renderer.render_by_line(provider);
        });
        if let Ok(event) = rx.try_recv() {
            match event {
                ButtonEvent::Clicked => {
                    info!("Clicked");
                    window.dispatch_event(WindowEvent::KeyPressed {
                        text: Key::Tab.into(),
                    });
                    thread::sleep(Duration::from_millis(30));
                    window.dispatch_event(WindowEvent::KeyReleased {
                        text: Key::Tab.into(),
                    });
                }
                ButtonEvent::DoubleClicked => {
                    window.dispatch_event(WindowEvent::KeyPressed {
                        text: Key::Return.into(),
                    });
                    thread::sleep(Duration::from_millis(30));
                    window.dispatch_event(WindowEvent::KeyReleased {
                        text: Key::Return.into(),
                    });
                }
                ButtonEvent::TripleClicked => {
                    info!("TripleClicked");
                }
                ButtonEvent::Pressed(dur) => {
                    info!("Pressed {:?}", dur);
                }
                ButtonEvent::Released(dur) => {
                    info!("Released {:?}", dur);

                }
            }
        }
        if window.has_active_animations() {
            continue
        }
    }

}
