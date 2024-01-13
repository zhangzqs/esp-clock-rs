use android_activity::{AndroidApp, InputStatus, MainEvent, PollEvent};
use android_logger::Filter;
use button_driver::{Button, ButtonConfig, PinWrapper};
use embedded_graphics::{
    geometry::{Point, Size},
    pixelcolor::Rgb888,
    primitives::Rectangle,
};
use embedded_graphics_group::{DisplayGroup, LogicalDisplay};
use embedded_svc::http::{server::Handler, Method};
use embedded_tone::RawTonePlayer;
use log::{debug, info};
use slint::{Image, SharedPixelBuffer};
use slint_app::{BootState, EvilApple, LEDController, MyApp, MyAppDeps};
use std::{
    marker::PhantomData,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    thread,
};

use desktop_svc::storage::KVStorage;
use i_slint_backend_android_activity::AndroidPlatform;
use std::rc::Rc;
use std::time::Duration;

mod interface_impl;
use interface_impl::*;
#[derive(Clone)]
struct MyButtonPin(Rc<AtomicBool>);

impl PinWrapper for MyButtonPin {
    fn is_high(&self) -> bool {
        self.0.load(Ordering::Relaxed)
    }
}

#[no_mangle]
fn android_main(app: AndroidApp) {
    android_logger::init_once(android_logger::Config::default().with_min_level(log::Level::Debug));

    let kv = KVStorage::new({
        let mut db_path = app.internal_data_path().unwrap();
        db_path.push("storage.db");
        db_path
    })
    .unwrap();

    slint::platform::set_platform(Box::new(AndroidPlatform::new(app))).unwrap();
    info!("Android Main");

    let buf = Arc::new(Mutex::new(SharedPixelBuffer::<slint::Rgb8Pixel>::new(
        240, 240,
    )));
    let physical_display = Arc::new(Mutex::new(
        embedded_graphics_slint_image_buf::SlintPixelBufferDrawTarget { buf: buf.clone() },
    ));
    let display_group = Arc::new(Mutex::new(DisplayGroup::new(physical_display.clone(), 2)));
    let mock_main_logical_display = LogicalDisplay::new(
        display_group.clone(),
        Rectangle::new(Point::new(0, 0), Size::new(240, 240)),
    );
    let mock_main_logical_display_id = mock_main_logical_display.lock().unwrap().get_id() as isize;
    display_group
        .lock()
        .unwrap()
        .switch_to_logical_display(mock_main_logical_display_id);

    let app = Rc::new(MyApp::new(MyAppDeps {
        system: MockSystem,
        display_group: display_group.clone(),
        player: RodioPlayer::new(),
        eval_apple: MockEvilApple,
        screen_brightness_controller: MockLEDController::new(),
        blue_led: MockLEDController::new(),
        http_client_builder: PhantomData::<HttpClientBuilder>,
        http_server_builder: PhantomData::<HttpServerBuilder>,
        raw_storage: kv,
    }));

    let u = app.get_app_window();
    let display_group_timer = slint::Timer::default();
    display_group_timer.start(
        slint::TimerMode::Repeated,
        Duration::from_millis(100),
        move || {
            let mut display_group = display_group.lock().unwrap();
            let show_main = display_group.get_current_active_display_index() == 0;
            drop(display_group);
            if let Some(ui) = u.upgrade() {
                ui.set_show_external_display(!show_main);
                if !show_main {
                    ui.set_external_display_image(Image::from_rgb8(buf.lock().unwrap().clone()));
                }
            }
        },
    );

    let u = app.get_app_window();
    thread::spawn(move || {
        u.upgrade_in_event_loop(|ui| {
            ui.invoke_set_boot_state(BootState::Booting);
        })
        .unwrap();
        thread::sleep(Duration::from_secs(1));
        u.upgrade_in_event_loop(|ui| {
            ui.invoke_set_boot_state(BootState::Connecting);
        })
        .unwrap();
        thread::sleep(Duration::from_secs(5));
        u.upgrade_in_event_loop(|ui| {
            ui.invoke_set_boot_state(BootState::BootSuccess);
        })
        .unwrap();
        thread::sleep(Duration::from_secs(1));
        u.upgrade_in_event_loop(|ui| {
            ui.invoke_set_boot_state(BootState::Finished);
        })
        .unwrap();
    });

    // 分发按键事件
    // 假设代表按键状态，默认为松开，值为false
    let button_state = Rc::new(AtomicBool::new(false));
    let mut button = Button::new(
        MyButtonPin(button_state.clone()),
        ButtonConfig {
            mode: button_driver::Mode::PullDown, // 当按键松开时，是低电平
            ..Default::default()
        },
    );

    let button_event_timer = slint::Timer::default();
    let u = app.get_app_window();
    button_event_timer.start(
        slint::TimerMode::Repeated,
        Duration::from_millis(10),
        move || {
            button.tick();
            if button.clicks() > 0 {
                let clicks = button.clicks();
                debug!("Clicks: {}", clicks);
                if let Some(ui) = u.upgrade() {
                    ui.invoke_on_one_button_clicks(clicks as i32);
                }
            } else if let Some(dur) = button.current_holding_time() {
                debug!("Held for {dur:?}");
                if let Some(ui) = u.upgrade() {
                    ui.invoke_on_one_button_long_pressed_holding(dur.as_millis() as i64);
                }
            } else if let Some(dur) = button.held_time() {
                debug!("Total holding time {dur:?}");
                if let Some(ui) = u.upgrade() {
                    ui.invoke_on_one_button_long_pressed_held(dur.as_millis() as i64);
                }
            }
            button.reset();
        },
    );

    let button_state = button_state.clone();
    if let Some(ui) = app.get_app_window().upgrade() {
        ui.on_touch_area_pointer_event(move |e| {
            let kind = format!("{}", e.kind);
            match kind.as_str() {
                "down" => {
                    button_state.store(true, Ordering::Relaxed);
                }
                "up" => {
                    button_state.store(false, Ordering::Relaxed);
                }
                _ => {}
            }
        });
    }

    app.run().unwrap();
}
