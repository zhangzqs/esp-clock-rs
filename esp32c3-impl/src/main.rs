use display_interface_spi::SPIInterface;
use embedded_graphics::{pixelcolor::Rgb565, prelude::*, primitives::Rectangle, text::Text};
use embedded_hal::spi::MODE_3;
use embedded_software_slint_backend::EmbeddedSoftwarePlatform;
use esp_idf_hal::{
    delay::{Ets, FreeRtos},
    gpio::{AnyIOPin, Gpio8, PinDriver},
    ledc::{config::TimerConfig, LedcDriver, LedcTimer, LedcTimerDriver},
    prelude::*,
    spi::{config::Config, Dma, SpiDeviceDriver, SpiDriverConfig},
};
use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    http::client::EspHttpConnection,
    nvs::{EspDefaultNvsPartition, EspNvs},
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
use slint_app::BootState;

use std::time::Duration;
use std::{cell::RefCell, thread};
use std::{cell::UnsafeCell, sync::Mutex};
use std::{rc::Rc, sync::Arc};

use crate::wifi::connect_to_wifi;

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

struct MyConn(pub EspHttpConnection);

unsafe impl Send for MyConn {}

fn main() -> anyhow::Result<()> {
    esp_idf_sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();

    let peripherals = Peripherals::take().unwrap();

    // 所有引脚定义
    let btn_pin = PinDriver::input(peripherals.pins.gpio9)?;
    let cs = PinDriver::output(peripherals.pins.gpio5)?;
    let dc = PinDriver::output(peripherals.pins.gpio4)?;
    let rst = PinDriver::output(peripherals.pins.gpio8)?;

    // 初始化SPI引脚
    let mut delay = FreeRtos;
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
    info!("SPI init done");

    // 初始化ledc控制器
    let mut led = LedcDriver::new(
        peripherals.ledc.channel0,
        LedcTimerDriver::new(
            peripherals.ledc.timer0,
            &TimerConfig::new().frequency(25.kHz().into()),
        )
        .unwrap(),
        peripherals.pins.gpio2,
    )
    .unwrap();

    thread::spawn(move || {
        let max_duty = led.get_max_duty();
        for numerator in (0..256).chain((0..256).rev()).cycle() {
            led.set_duty(max_duty * numerator / 256).unwrap();
            thread::sleep(Duration::from_millis(10));
        }
    });

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

    // 显示白色表示初始化完成
    display
        .fill_solid(
            &Rectangle {
                top_left: Point::new(0, 0),
                size: Size::new(240, 240),
            },
            Rgb565::WHITE,
        )
        .unwrap();

    // 连接wifi并NTP校时
    let nvs = EspDefaultNvsPartition::take()?;

    let mut nvs_a = EspNvs::new(nvs.clone(), "test_ns", true)?;
    let cnt = nvs_a.get_i32("test_key").unwrap().unwrap_or(0);
    info!("cnt: {}", cnt);
    nvs_a.set_i32("test_key", cnt + 1).unwrap();
    let sysloop = EspSystemEventLoop::take()?;

    let fps = Rc::new(RefCell::new(0));
    let fps_ref = fps.clone();
    slint::platform::set_platform(Box::new(EmbeddedSoftwarePlatform::new(
        Rc::new(RefCell::new(display)),
        Some(move |redraw| {
            if redraw {
                *fps_ref.borrow_mut() += 1;
            }
            Ok(())
        }),
    )))
    .unwrap();

    let app = Rc::new(slint_app::MyApp::new(slint_app::MyAppDeps {
        http_conn: EspHttpConnection::new(&esp_idf_svc::http::client::Configuration {
            timeout: Some(Duration::from_secs(60)),
            ..Default::default()
        })?,
    }));

    let wifi = Arc::new(Mutex::new(None));
    let sntp = Arc::new(Mutex::new(None));

    let w1 = wifi.clone();
    let s1 = sntp.clone();

    app.set_boot_state(BootState::Booting);
    let u = app.get_app_window();
    thread::Builder::new().stack_size(4096).spawn(move || {
        u.upgrade_in_event_loop(|ui| {
            ui.invoke_set_boot_state(BootState::Connecting);
        })
        .unwrap();
        let _wifi = connect_to_wifi(
            MY_CONFIG.wifi_ssid,
            MY_CONFIG.wifi_password,
            peripherals.modem,
            sysloop,
            Some(nvs),
        );

        if _wifi.is_err() {
            let err = _wifi.err().unwrap();
            error!("wifi err: {:?}", err);
            u.upgrade_in_event_loop(|ui| {
                ui.invoke_set_boot_state(BootState::BootFailed);
            })
            .unwrap();
            return;
        }
        w1.lock().unwrap().replace(_wifi.unwrap());

        u.upgrade_in_event_loop(|ui| {
            ui.invoke_set_boot_state(BootState::BootSuccess);
        })
        .unwrap();

        let _sntp = EspSntp::new(&SntpConf {
            servers: [MY_CONFIG.ntp_server],
            sync_mode: SyncMode::Immediate,
            operating_mode: OperatingMode::Poll,
        });
        s1.lock().unwrap().replace(_sntp.unwrap());

        thread::sleep(Duration::from_secs(1));
        u.upgrade_in_event_loop(|ui| {
            ui.invoke_set_boot_state(BootState::Finished);
        })
        .unwrap();
    })?;

    // 按键驱动
    let ui = app.get_app_window();
    thread::spawn(move || {
        let mut button = button_driver::Button::new(btn_pin, Default::default());
        loop {
            let ui = ui.clone();
            button.tick();
            if button.clicks() > 0 {
                let clicks = button.clicks();
                info!("Clicks: {}", clicks);
                ui.upgrade_in_event_loop(move |ui| {
                    ui.invoke_on_one_button_clicks(clicks as i32);
                })
                .unwrap();
            } else if let Some(dur) = button.current_holding_time() {
                info!("Held for {dur:?}");
                ui.upgrade_in_event_loop(move |ui| {
                    ui.invoke_on_one_button_long_pressed_holding(dur.as_millis() as i64);
                })
                .unwrap();
            } else if let Some(dur) = button.held_time() {
                info!("Total holding time {dur:?}");
                ui.upgrade_in_event_loop(move |ui| {
                    ui.invoke_on_one_button_long_pressed_held(dur.as_millis() as i64);
                })
                .unwrap();
            }
            button.reset();
            thread::sleep(Duration::from_millis(10));
        }
    });

    // fps计数器
    let u = app.get_app_window();
    let frame_timer = slint::Timer::default();
    frame_timer.start(
        slint::TimerMode::Repeated,
        Duration::from_secs(1),
        move || {
            u.upgrade().and_then(|ui| {
                ui.set_fps(*fps.borrow());
                Some(())
            });
            *fps.borrow_mut() = 0;
        },
    );

    let free_mem_timer = slint::Timer::default();
    free_mem_timer.start(
        slint::TimerMode::Repeated,
        Duration::from_secs(1),
        move || unsafe {
            let free = esp_idf_sys::esp_get_free_heap_size();
            info!("free heap: {}", free);
        },
    );
    slint::run_event_loop().map_err(|e| anyhow::anyhow!("{:?}", e))?;
    Ok(())
}
