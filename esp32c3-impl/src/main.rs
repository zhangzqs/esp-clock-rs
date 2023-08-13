use display_interface_spi::SPIInterface;
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
use slint::platform::software_renderer::{MinimalSoftwareWindow, Rgb565Pixel};

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

    let nvs = EspDefaultNvsPartition::take()?;
    let sysloop = EspSystemEventLoop::take()?;
    let _wifi = connect_to_wifi(MY_CONFIG.wifi_ssid, MY_CONFIG.wifi_password, peripherals.modem, sysloop, Some(nvs))?;

    let _sntp = EspSntp::new(&SntpConf {
        servers: [MY_CONFIG.ntp_server],
        ..Default::default()
    })?;

    let cs = PinDriver::output(peripherals.pins.gpio5)?;
    let dc = PinDriver::output(peripherals.pins.gpio4)?;
    let rst = PinDriver::output(peripherals.pins.gpio8)?;

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
    let di = SPIInterface::new(spi, dc, cs);
    let mut display = Builder::st7789(di)
        .with_display_size(240, 240)
        .with_framebuffer_size(240, 240)
        .with_orientation(Orientation::Portrait(false))
        .with_invert_colors(ColorInversion::Inverted)
        .init(&mut delay, Some(rst))
        .unwrap();
    info!("display init done");

    let window: Rc<MinimalSoftwareWindow> = MinimalSoftwareWindow::new(Default::default());

    slint::platform::set_platform(Box::new(draw::MyPlatform {
        window: window.clone(),
    }))
    .unwrap();
    let _ui = slint_app::create_app();

    let line_buffer = &mut [Rgb565Pixel::default(); 240];

    loop {
        thread::sleep(Duration::from_millis(16));

        slint::platform::update_timers_and_animations();
        window.draw_if_needed(|renderer| {
            let provider = draw::MyLineBufferProvider {
                display: &mut display,
                line_buffer: line_buffer,
            };
            renderer.render_by_line(provider);
        });
        if window.has_active_animations() {
            continue;
        }
    }
}
