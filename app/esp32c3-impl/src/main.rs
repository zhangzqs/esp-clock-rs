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
use embedded_graphics_mux::{DisplayMux, LogicalDisplay};
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
use peripherals::SystemPeripherals;

mod node;
mod peripherals;

fn main() -> anyhow::Result<()> {
    esp_idf_sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();
    esp_idf_svc::log::set_target_level("esp32c3_impl", log::LevelFilter::Debug)?;
    esp_idf_svc::log::set_target_level("app_core", log::LevelFilter::Debug)?;

    // 所有引脚定义
    let peripherals = SystemPeripherals::take();
    let cs = PinDriver::output(peripherals.display.cs)?;
    let dc = PinDriver::output(peripherals.display.control.dc)?;
    let rst = PinDriver::output(peripherals.display.control.rst)?;

    // 初始化SPI引脚
    let spi = SpiDeviceDriver::new_single(
        peripherals.display.spi,
        peripherals.display.sclk,
        peripherals.display.sdo,
        Option::<AnyIOPin>::None,
        Option::<AnyIOPin>::None,
        &SpiDriverConfig::default().dma(Dma::Auto(4096)),
        &Config::default()
            .baudrate(60.MHz().into())
            .data_mode(esp_idf_hal::spi::config::MODE_3),
    )?;

    // 初始化显示屏驱动
    let display = Builder::st7789(SPIInterface::new(spi, dc, cs))
        .with_display_size(240, 240)
        .with_framebuffer_size(240, 240)
        .with_orientation(Orientation::Portrait(false))
        .with_invert_colors(ColorInversion::Inverted)
        .init(&mut FreeRtos, Some(rst))
        .unwrap();

    // 设置底部灯为关闭
    if let Some(ledc) = peripherals.board_led {
        let mut ledc_driver = LedcDriver::new(
            ledc.ledc_channel,
            LedcTimerDriver::new(
                ledc.ledc_timer,
                &TimerConfig::new().frequency(25.kHz().into()),
            )
            .unwrap(),
            ledc.pin,
        )
        .unwrap();
        ledc_driver.set_duty(0).unwrap();
    }

    // 设置屏幕背光亮度为100%
    if let Some(ledc) = peripherals.display.control.backlight {
        let mut ledc_driver = LedcDriver::new(
            ledc.ledc_channel,
            LedcTimerDriver::new(
                ledc.ledc_timer,
                &TimerConfig::new().frequency(25.kHz().into()),
            )
            .unwrap(),
            ledc.pin,
        )
        .unwrap();
        ledc_driver.set_duty(ledc_driver.get_max_duty()).unwrap();
    }
    let phy_display = Rc::new(RefCell::new(display));
    let display_mux = Rc::new(RefCell::new(DisplayMux::new(phy_display, 4)));
    let slint_logic_display = LogicalDisplay::new(display_mux.clone());
    let slint_logic_display_id = slint_logic_display.borrow().get_id() as isize;
    display_mux.borrow_mut().switch_to(slint_logic_display_id);

    let nvs = EspDefaultNvsPartition::take()?;

    let frame_counter = Arc::new(AtomicUsize::new(0));
    let platform = embedded_software_slint_backend::MySoftwarePlatform::new(
        slint_logic_display,
        Some({
            let fps_ref = frame_counter.clone();
            move |_| {
                fps_ref.fetch_add(1, Ordering::SeqCst);
                Ok(())
            }
        }),
    );
    slint::platform::set_platform(Box::new(platform)).map_err(|e| anyhow::anyhow!("{:?}", e))?;

    let sche = get_scheduler();
    if let Some(btn) = peripherals.button {
        let btn_pin = PinDriver::input(btn)?;
        sche.register_node(OneButtonService::new(btn_pin));
    }
    if let Some(buzzer) = peripherals.buzzer {
        let beep_tx = TxRmtDriver::new(
            buzzer.rmt_channel,
            buzzer.pin,
            &TransmitConfig::new().looping(Loop::Endless),
        )
        .unwrap();
        sche.register_node(BuzzerService::new(beep_tx));
    }

    sche.register_node(SystemService::new(frame_counter));
    sche.register_node(WiFiService::new(
        nvs.clone(),
        EspSystemEventLoop::take().unwrap(),
        peripherals.modem,
    ));
    sche.register_node(SntpService::new());
    sche.register_node(HttpClientService::new());
    sche.register_node(NvsStorageService::new(nvs.clone()));
    sche.register_node(HttpServerService::new());
    sche.register_node(CanvasView::new(display_mux.clone()));
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
