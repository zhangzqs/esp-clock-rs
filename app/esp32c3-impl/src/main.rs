#![feature(iterator_try_collect)]

use std::{
    cell::RefCell,
    rc::Rc,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    time::Duration,
};

use crate::node::*;
use app_core::get_scheduler;
use display_interface_spi::SPIInterface;
use embedded_hal::spi::MODE_3;
use esp_idf_hal::{
    delay::FreeRtos,
    gpio::{AnyIOPin, PinDriver},
    ledc::{config::TimerConfig, LedcDriver, LedcTimerDriver},
    prelude::*,
    rmt::{
        config::{Loop, TransmitConfig},
        TxRmtDriver,
    },
    spi::{config::Config, Dma, SpiDeviceDriver, SpiDriverConfig},
};
use esp_idf_svc::{eventloop::EspSystemEventLoop, nvs::EspDefaultNvsPartition};
use esp_idf_sys as _;
use mipidsi::{Builder, ColorInversion, Orientation};

mod node;

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

    let nvs = EspDefaultNvsPartition::take()?;

    let frame_counter = Arc::new(AtomicUsize::new(0));
    let platform = embedded_software_slint_backend::MySoftwarePlatform::new(
        Rc::new(RefCell::new(display)),
        Some({
            let fps_ref = frame_counter.clone();
            move |_| {
                fps_ref.fetch_add(1, Ordering::SeqCst);
                Ok(())
            }
        }),
    );
    slint::platform::set_platform(Box::new(platform)).map_err(|e| anyhow::anyhow!("{:?}", e))?;

    let btn_pin = PinDriver::input(peripherals.pins.gpio9)?;
    let beep_tx = TxRmtDriver::new(
        peripherals.rmt.channel0,
        peripherals.pins.gpio0,
        &TransmitConfig::new().looping(Loop::Endless),
    )
    .unwrap();

    let sche = get_scheduler();
    sche.register_node(OneButtonService::new(btn_pin));
    sche.register_node(SystemService::new(frame_counter));
    sche.register_node(WiFiService::new(
        nvs.clone(),
        EspSystemEventLoop::take().unwrap(),
        peripherals.modem,
    ));
    sche.register_node(SntpService::new());
    sche.register_node(HttpClientService::new());
    sche.register_node(NvsStorageService::new(nvs.clone()));
    sche.register_node(BuzzerService::new(beep_tx));
    sche.register_node(HttpServerService::new());
    let sche_timer = slint::Timer::default();
    sche_timer.start(
        slint::TimerMode::Repeated,
        Duration::from_millis(16),
        move || {
            sche.schedule_once();
        },
    );

    slint::run_event_loop().map_err(|e| anyhow::anyhow!("{:?}", e))?;
    Ok(())
}
