use display_interface_spi::SPIInterface;
use embedded_graphics::{text::Text, prelude::*};
use embedded_hal::spi::MODE_3;
use esp_idf_hal::{
    delay::Ets,
    gpio::{AnyIOPin, Gpio8, PinDriver},
    prelude::*,
    spi::{Dma, SpiDeviceDriver, SpiDriverConfig, config::Config},
};
use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    nvs::EspDefaultNvsPartition,
    sntp::{EspSntp, SntpConf},
};
use esp_idf_sys as _;
use log::*;
use mipidsi::{Builder, ColorInversion, Orientation};
use slint::{platform::software_renderer::{MinimalSoftwareWindow, Rgb565Pixel}, SharedString, ComponentHandle};
use slint_app::DateTime;

use std::rc::Rc;
use std::thread;
use std::time::Duration;

use crate::wifi::connect_to_wifi;

mod draw;
mod wifi;
#[toml_cfg::toml_config]
pub struct MyConfig {
    #[default("")]
    wifi_ssid: &'static str,
    #[default("")]
    wifi_password: &'static str,
    #[default("ntp.aliyun.com")]
    ntp_server: &'static str,
}

fn main() -> anyhow::Result<()> {
    esp_idf_sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take().unwrap();

    // 所有引脚定义
    let mut led_pin = PinDriver::output(peripherals.pins.gpio2)?;
    let btn_pin = PinDriver::input(peripherals.pins.gpio9)?;
    let cs = PinDriver::output(peripherals.pins.gpio5)?;
    let dc = PinDriver::output(peripherals.pins.gpio4)?;
    let rst = PinDriver::output(peripherals.pins.gpio8)?;

    // 初始化SPI引脚
    let mut delay = Ets;
    let spi = SpiDeviceDriver::new_single(
        peripherals.spi2,
        peripherals.pins.gpio6,
        peripherals.pins.gpio7,
        Option::<AnyIOPin>::None,
        Option::<AnyIOPin>::None,
        &SpiDriverConfig::default().dma(Dma::Auto(512)),
        &Config::default()
            .baudrate(80.MHz().into())
            .data_mode(MODE_3),
    )?;
    info!("SPI init done");

    // 初始化显示屏驱动
    let di = SPIInterface::new(spi, dc, cs);
    let mut display = Builder::st7789(di)
        .with_display_size(240, 240)
        .with_framebuffer_size(240, 240)
        .with_orientation(Orientation::Portrait(false))
        .with_invert_colors(ColorInversion::Inverted)
        .init(&mut delay, Some(rst))
        .unwrap();
    info!("display init done");

    // 按键驱动
    let mut btn = button_driver::Button::new(btn_pin, Default::default());

    // 初始化slint
    let window: Rc<MinimalSoftwareWindow> = MinimalSoftwareWindow::new(Default::default());
    slint::platform::set_platform(Box::new(draw::MyPlatform {window: window.clone()}))
    .unwrap();

    let app = slint_app::MyApp::new();
    let line_buffer = &mut [Rgb565Pixel::default(); 240];
    
    // 主线程循环
    loop {
        slint::platform::update_timers_and_animations();
        window.draw_if_needed(|renderer| {
            let provider = draw::MyLineBufferProvider {
                display: &mut display,
                line_buffer: line_buffer,
            };
            renderer.render_by_line(provider);
        });
        // 如果有未完成的动画计算，则继续完成动画计算
        if window.has_active_animations() {
            continue;
        }

        // 按键事件处理
        btn.tick();
        if btn.is_clicked() {
            app.go_to_next_page();
        } else if btn.is_double_clicked() {
            app.go_to_prev_page();
        } else if let Some(t) = btn.current_holding_time() {
            if t > Duration::from_secs(3) {
                app.go_to_home_page();
            }
        }
        btn.reset();

        // 这段sleep可以使得IDLE任务有时间喂狗，否则容易触发看门狗导致重启
        thread::sleep(Duration::from_millis(16));
    }
}
