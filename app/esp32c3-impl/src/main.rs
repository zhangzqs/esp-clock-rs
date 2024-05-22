use std::{cell::RefCell, rc::Rc, time::Duration};

use app_core::{get_scheduler, proto::*};
use button_driver::Button;
use display_interface_spi::SPIInterface;
use embedded_hal::spi::MODE_3;
use esp_idf_hal::{
    delay::FreeRtos,
    gpio::{AnyIOPin, Input, Pin, PinDriver},
    ledc::{config::TimerConfig, LedcDriver, LedcTimerDriver},
    prelude::*,
    spi::{config::Config, Dma, SpiDeviceDriver, SpiDriverConfig},
};
use esp_idf_sys as _;
use log::info;
use mipidsi::{Builder, ColorInversion, Orientation};

struct EspOneButton<'a, P: Pin> {
    button: Rc<RefCell<Button<PinDriver<'a, P, Input>, button_driver::DefaultPlatform>>>,
    timer: slint::Timer,
}

impl<'a, P: Pin> EspOneButton<'a, P> {
    fn new(pin: PinDriver<'a, P, Input>) -> Self {
        let button = button_driver::Button::new(pin, Default::default());
        Self {
            button: Rc::new(RefCell::new(button)),
            timer: slint::Timer::default(),
        }
    }
}

impl<'a: 'static, P: Pin> Node for EspOneButton<'a, P> {
    fn node_name(&self) -> NodeName {
        NodeName::Other("EspOneButton")
    }

    fn handle_message(
        &mut self,
        ctx: Rc<dyn Context>,
        _from: NodeName,
        _to: MessageTo,
        msg: MessageWithHeader,
    ) -> HandleResult {
        if let Message::Lifecycle(LifecycleMessage::Init) = msg.body {
            let button = self.button.clone();
            self.timer.start(
                slint::TimerMode::Repeated,
                Duration::from_millis(20),
                move || {
                    let mut button = button.borrow_mut();
                    button.tick();

                    if button.clicks() > 0 {
                        let clicks = button.clicks();
                        if clicks == 1 {
                            ctx.send_message(
                                MessageTo::Broadcast,
                                Message::OneButton(OneButtonMessage::Click),
                            );
                        } else {
                            ctx.send_message(
                                MessageTo::Broadcast,
                                Message::OneButton(OneButtonMessage::Clicks(clicks)),
                            );
                        }
                    } else if let Some(dur) = button.current_holding_time() {
                        info!("Held for {dur:?}");
                        ctx.send_message(
                            MessageTo::Broadcast,
                            Message::OneButton(OneButtonMessage::LongPressHolding(dur)),
                        );
                    } else if let Some(dur) = button.held_time() {
                        info!("Total holding time {dur:?}");
                        ctx.send_message(
                            MessageTo::Broadcast,
                            Message::OneButton(OneButtonMessage::LongPressHeld(dur)),
                        );
                    }
                    button.reset();
                },
            );
        }
        HandleResult::Discard
    }
}

fn main() -> anyhow::Result<()> {
    esp_idf_sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();
    esp_idf_svc::log::set_target_level("esp32c3_impl", log::LevelFilter::Debug)?;
    esp_idf_svc::log::set_target_level("app_core", log::LevelFilter::Debug)?;

    let peripherals = Peripherals::take().unwrap();
    // 所有引脚定义
    let cs = PinDriver::output(peripherals.pins.gpio5)?;
    let dc = PinDriver::output(peripherals.pins.gpio4)?;
    let rst = PinDriver::output(peripherals.pins.gpio8)?;

    // 初始化SPI引脚
    let spi = SpiDeviceDriver::new_single(
        peripherals.spi2,
        peripherals.pins.gpio6,
        peripherals.pins.gpio7,
        Option::<AnyIOPin>::None,
        Option::<AnyIOPin>::None,
        &SpiDriverConfig::default().dma(Dma::Auto(4096)),
        &Config::default()
            .baudrate(80.MHz().into())
            .data_mode(MODE_3),
    )?;

    // 设置底部灯为关闭
    let mut blue_led = LedcDriver::new(
        peripherals.ledc.channel0,
        LedcTimerDriver::new(
            peripherals.ledc.timer0,
            &TimerConfig::new().frequency(25.kHz().into()),
        )
        .unwrap(),
        peripherals.pins.gpio2,
    )
    .unwrap();
    blue_led.set_duty(0).unwrap();

    // 设置屏幕背光亮度为100%
    let mut screen_ledc = LedcDriver::new(
        peripherals.ledc.channel1,
        LedcTimerDriver::new(
            peripherals.ledc.timer1,
            &TimerConfig::new().frequency(25.kHz().into()),
        )
        .unwrap(),
        peripherals.pins.gpio10,
    )
    .unwrap();
    screen_ledc.set_duty(screen_ledc.get_max_duty()).unwrap();

    // 初始化显示屏驱动
    let display = Builder::st7789(SPIInterface::new(spi, dc, cs))
        .with_display_size(240, 240)
        .with_framebuffer_size(240, 240)
        .with_orientation(Orientation::Portrait(false))
        .with_invert_colors(ColorInversion::Inverted)
        .init(&mut FreeRtos, Some(rst))
        .unwrap();

    let platform = embedded_software_slint_backend::MySoftwarePlatform::new(
        Rc::new(RefCell::new(display)),
        Some(|_| Ok(())),
    );
    slint::platform::set_platform(Box::new(platform)).map_err(|e| anyhow::anyhow!("{:?}", e))?;

    let btn_pin = PinDriver::input(peripherals.pins.gpio9)?;

    let one_butten_node = EspOneButton::new(btn_pin);
    let mut sche = get_scheduler();
    sche.register_node(one_butten_node);
    let sche_timer = slint::Timer::default();
    sche_timer.start(
        slint::TimerMode::Repeated,
        Duration::from_millis(20),
        move || {
            sche.schedule_once();
        },
    );

    slint::run_event_loop().map_err(|e| anyhow::anyhow!("{:?}", e))?;
    Ok(())
}
