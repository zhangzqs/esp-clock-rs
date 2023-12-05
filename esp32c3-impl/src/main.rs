use display_interface_spi::SPIInterface;
use embedded_graphics::{pixelcolor::Rgb565, prelude::*, primitives::Rectangle};
use embedded_graphics_group::{DisplayGroup, LogicalDisplay};
use embedded_hal::spi::MODE_3;
use embedded_software_slint_backend::{EmbeddedSoftwarePlatform, RGB565PixelColorAdapter};
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
use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    nvs::{EspDefaultNvsPartition, EspNvs},
    sntp::{EspSntp, OperatingMode, SntpConf, SyncMode},
};
use esp_idf_sys as _;
use log::*;
use mipidsi::{Builder, ColorInversion, Orientation};
use slint_app::{BootState, System};
use std::{
    cell::RefCell,
    rc::Rc,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

mod connection;
mod evil_apple;
mod led_controller;
mod player;
mod system;
mod wifi;

use crate::{
    led_controller::EspLEDController, player::EspBeepPlayer, system::EspSystem,
    wifi::connect_to_wifi,
};

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

    let sys = EspSystem;
    info!("start free heap: {}", sys.get_free_heap_size());
    info!("start largest free block: {}", sys.get_largest_free_block());

    let peripherals = Peripherals::take().unwrap();
    // 所有引脚定义
    let btn_pin = PinDriver::input(peripherals.pins.gpio9)?;
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
        // .write_only(true)
        // .queue_size(128),
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

    // 设置屏幕背光亮度为33%
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

    let beep_tx = TxRmtDriver::new(
        peripherals.rmt.channel0,
        peripherals.pins.gpio0,
        &TransmitConfig::new().looping(Loop::Endless),
    )
    .unwrap();

    // 初始化显示屏驱动
    let physical_display = Arc::new(Mutex::new(
        Builder::st7789(SPIInterface::new(spi, dc, cs))
            .with_display_size(240, 240)
            .with_framebuffer_size(240, 240)
            .with_orientation(Orientation::Portrait(false))
            .with_invert_colors(ColorInversion::Inverted)
            .init(&mut FreeRtos, Some(rst))
            .unwrap(),
    ));

    let display_group = Arc::new(Mutex::new(DisplayGroup::new(physical_display.clone(), 2)));

    let nvs = EspDefaultNvsPartition::take()?;
    let nvs_a = EspNvs::new(nvs.clone(), "test_ns", true)?;
    let cnt = nvs_a.get_i32("test_key").unwrap().unwrap_or(0);
    info!("cnt: {}", cnt);
    nvs_a.set_i32("test_key", cnt + 1).unwrap();

    info!("nvs init done");
    info!("free heap: {}", sys.get_free_heap_size());

    let fps = Rc::new(RefCell::new(0));
    {
        let slint_display = LogicalDisplay::new(
            display_group.clone(),
            Rectangle {
                top_left: Point::new(0, 0),
                size: Size::new(240, 240),
            },
        );
        let slint_display_id = slint_display.lock().unwrap().get_id() as isize;
        display_group
            .lock()
            .unwrap()
            .switch_to_logical_display(slint_display_id);
        // 显示白色表示初始化完成
        slint_display
            .lock()
            .unwrap()
            .fill_solid(
                &Rectangle {
                    top_left: Point::new(0, 0),
                    size: Size::new(240, 240),
                },
                Rgb565::WHITE,
            )
            .unwrap();
        let fps_ref = fps.clone();
        slint::platform::set_platform(Box::new(EmbeddedSoftwarePlatform::<
            _,
            _,
            _,
            _,
            RGB565PixelColorAdapter,
        >::new(
            slint_display.clone(),
            Some(move |redraw| {
                if redraw {
                    *fps_ref.borrow_mut() += 1;
                }
                Ok(())
            }),
        )))
        .unwrap();
        info!("slint platform init done");
        info!("free heap: {}", sys.get_free_heap_size());
        info!("largest free block: {}", sys.get_largest_free_block());
    }

    let app = slint_app::MyApp::new(slint_app::MyAppDeps {
        http_conn: connection::MyConnection::new(),
        system: EspSystem,
        display_group,
        player: EspBeepPlayer::new(beep_tx),
        eval_apple: evil_apple::EvilAppleBLEImpl,
        screen_brightness_controller: EspLEDController::new(screen_ledc),
        blue_led: EspLEDController::new(blue_led),
    });
    info!("slint app init done");
    info!("free heap: {}", sys.get_free_heap_size());
    info!("largest free block: {}", sys.get_largest_free_block());

    // 连接wifi并NTP校时
    let sysloop = EspSystemEventLoop::take()?;
    let wifi = Arc::new(Mutex::new(None));
    let sntp = Arc::new(Mutex::new(None));

    let w1 = wifi.clone();
    let s1 = sntp.clone();

    if let Some(ui) = app.get_app_window().upgrade() {
        ui.invoke_set_boot_state(BootState::Booting);
    }
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
        info!("try wifi connect");
        info!("free heap: {}", sys.get_free_heap_size());
        info!("largest free block: {}", sys.get_largest_free_block());

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
        info!("wifi connected");
        info!("free heap: {}", sys.get_free_heap_size());
        info!("largest free block: {}", sys.get_largest_free_block());

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
        info!("sntp init done");
        info!("free heap: {}", sys.get_free_heap_size());
        info!("largest free block: {}", sys.get_largest_free_block());

        u.upgrade_in_event_loop(|ui| {
            ui.invoke_set_boot_state(BootState::Finished);
        })
        .unwrap();
    })?;

    let sys = EspSystem;
    info!("wifi thread init done");
    info!("free heap: {}", sys.get_free_heap_size());
    info!("largest free block: {}", sys.get_largest_free_block());

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
    info!("button driver thread init done");
    info!("free heap: {}", sys.get_free_heap_size());
    info!("largest free block: {}", sys.get_largest_free_block());

    // 性能监视器
    let u = app.get_app_window();
    let perf_timer = slint::Timer::default();
    perf_timer.start(
        slint::TimerMode::Repeated,
        Duration::from_secs(1),
        move || {
            let sys = EspSystem;
            let free = sys.get_free_heap_size();
            let largest = sys.get_largest_free_block();
            let fps_ref = fps.clone();
            u.upgrade().map(move |ui| {
                ui.set_memory(free as i32);
                ui.set_largest_free_block(largest as i32);
                ui.set_fps(*fps_ref.borrow());
            });
            let fps_ref = fps.clone();
            *fps_ref.borrow_mut() = 0;
        },
    );
    info!("free mem timer init done");
    info!("free heap: {}", sys.get_free_heap_size());
    info!("largest free block: {}", sys.get_largest_free_block());

    // 放到新线程，防止阻塞看门狗
    thread::Builder::new()
        .stack_size(8 * 1024)
        .name("Slint UI".into())
        .spawn(|| slint::run_event_loop().map_err(|e| anyhow::anyhow!("{:?}", e)))
        .unwrap();
    loop {
        thread::sleep(Duration::from_millis(100));
    }
}
