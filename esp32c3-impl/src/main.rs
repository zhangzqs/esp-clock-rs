use display_interface_spi::SPIInterface;
use embedded_graphics::{pixelcolor::Rgb565, prelude::*, primitives::Rectangle, text::Text};
use embedded_hal::spi::MODE_3;
use esp_idf_hal::{
    delay::Ets,
    gpio::{AnyIOPin, Gpio8, PinDriver},
    prelude::*,
    spi::{config::Config, Dma, SpiDeviceDriver, SpiDriverConfig},
};
use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    http::client::EspHttpConnection,
    nvs::EspDefaultNvsPartition,
    sntp::{self, EspSntp, OperatingMode, SntpConf, SyncMode},
    systime::EspSystemTime,
};
use esp_idf_sys as _;
use log::*;
use mipidsi::{Builder, ColorInversion, Orientation};
use slint::{
    platform::software_renderer::{MinimalSoftwareWindow, Rgb565Pixel},
    ComponentHandle, SharedString,
};
use slint_app::DateTime;

use std::sync::Mutex;
use std::thread;
use std::time::Duration;
use std::{rc::Rc, sync::Arc};

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

    // 显示绿色表示初始化完成
    display
        .fill_solid(
            &Rectangle {
                top_left: Point::new(0, 0),
                size: Size::new(240, 240),
            },
            Rgb565::GREEN,
        )
        .unwrap();

    // 连接wifi并NTP校时
    let nvs = EspDefaultNvsPartition::take()?;
    let sysloop = EspSystemEventLoop::take()?;

    let wifi = Arc::new(Mutex::new(None));
    let sntp = Arc::new(Mutex::new(None));

    let w1 = wifi.clone();
    let s1 = sntp.clone();
    thread::Builder::new().stack_size(4096).spawn(move || {
        let _wifi = connect_to_wifi(
            MY_CONFIG.wifi_ssid,
            MY_CONFIG.wifi_password,
            peripherals.modem,
            sysloop,
            Some(nvs),
        );
        w1.lock().unwrap().replace(_wifi.unwrap());

        let _sntp = EspSntp::new(&SntpConf {
            servers: [MY_CONFIG.ntp_server],
            sync_mode: SyncMode::Immediate,
            operating_mode: OperatingMode::Poll,
        });
        s1.lock().unwrap().replace(_sntp.unwrap());
    })?;

    // 按键驱动
    let mut btn = button_driver::Button::new(btn_pin, Default::default());

    // 初始化slint
    let window: Rc<MinimalSoftwareWindow> = MinimalSoftwareWindow::new(Default::default());
    slint::platform::set_platform(Box::new(draw::MyPlatform {
        window: window.clone(),
    }))
    .unwrap();

    let mut conn = EspHttpConnection::new(&Default::default())?;
    let app = slint_app::MyApp::new(slint_app::MyAppDeps { http_conn: conn });
    let line_buffer = &mut [Rgb565Pixel::default(); 240];

    let mut has_boot = false;
    // 主线程循环
    loop {
        // 这段sleep可以使得IDLE任务有时间喂狗，否则容易触发看门狗导致重启
        thread::sleep(Duration::from_millis(16));

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

        // 如果未启动，且wifi已连接，则跳转首页
        if !has_boot && wifi.clone().lock().unwrap().is_some() {
            app.go_to_home_page();
            has_boot = true;
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
    }
}
