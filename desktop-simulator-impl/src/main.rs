use std::{
    cell::RefCell,
    env::set_var,
    marker::PhantomData,
    rc::Rc,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    thread,
    time::Duration,
};

use embedded_graphics::{pixelcolor::Rgb888, prelude::*, primitives::Rectangle};
use embedded_graphics_group::{DisplayGroup, LogicalDisplay};
use embedded_graphics_simulator::{
    sdl2::{Keycode, MouseButton},
    OutputSettingsBuilder, SimulatorDisplay, SimulatorEvent, Window,
};

use log::{debug, info};

use slint::Weak;
use slint_app::{BootState, MyApp, MyAppDeps};

use button_driver::{Button, ButtonConfig, PinWrapper};
use desktop_svc::storage::KVStorage;
use embedded_software_slint_backend::{EmbeddedSoftwarePlatform, RGB888PixelColorAdapter};

#[derive(Clone)]
struct MyButtonPin(Rc<AtomicBool>);

impl PinWrapper for MyButtonPin {
    fn is_high(&self) -> bool {
        self.0.load(Ordering::Relaxed)
    }
}

mod interface_impl;
use interface_impl::*;
fn main() -> anyhow::Result<()> {
    set_var("RUST_LOG", "debug");
    env_logger::init();
    info!("Starting desktop simulator");

    let kv = KVStorage::new("storage.db")?;

    let physical_display = Arc::new(Mutex::new(SimulatorDisplay::<Rgb888>::new(Size::new(
        240, 240,
    ))));
    let display_group = Arc::new(Mutex::new(DisplayGroup::new(physical_display.clone(), 2)));

    let fps = Rc::new(RefCell::new(0));
    {
        let fps_ref = fps.clone();
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

        let platform = EmbeddedSoftwarePlatform::<_, _, _, _, RGB888PixelColorAdapter>::new(
            slint_display,
            Some(Box::new(move |has_redraw| {
                if has_redraw {
                    *fps_ref.borrow_mut() += 1;
                }
                Ok(())
            })),
        );
        slint::platform::set_platform(Box::new(platform)).unwrap();
    }
    info!("platform has been set");

    let output_settings: embedded_graphics_simulator::OutputSettings =
        OutputSettingsBuilder::new().build();
    let mut window = Window::new("Desktop Simulator", &output_settings);
    info!("window has been created");

    let app_weak = Arc::new(Mutex::new(Weak::default()));

    let app = MyApp::new(MyAppDeps {
        system: MockSystem,
        display_group: display_group.clone(),
        player: RodioPlayer::new(),
        eval_apple: MockEvilApple,
        screen_brightness_controller: ScreenBrightnessController::new(app_weak.clone()),
        blue_led: MockLEDController::new(),
        http_client_builder: PhantomData::<HttpClientBuilder>,
        http_server_builder: PhantomData::<HttpServerBuilder>,
        raw_storage: kv,
    });
    *app_weak.lock().unwrap() = app.get_app_window();

    info!("app has been created");

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

    let u = app.get_app_window();
    let button_event_timer = slint::Timer::default();
    button_event_timer.start(
        slint::TimerMode::Repeated,
        Duration::from_millis(16),
        move || {
            {
                let physical_display_update_ref = physical_display.clone();
                let display = physical_display_update_ref.lock().unwrap();
                window.update(&display);
            }

            let button_state_ref = button_state.clone();
            for event in window.events() {
                match event {
                    SimulatorEvent::KeyUp { keycode, .. } => match keycode {
                        Keycode::Space => button_state_ref.store(false, Ordering::Relaxed),
                        _ => {}
                    },
                    SimulatorEvent::KeyDown { keycode, .. } => match keycode {
                        Keycode::Space => button_state_ref.store(true, Ordering::Relaxed),
                        _ => {}
                    },
                    SimulatorEvent::MouseButtonUp { mouse_btn, .. } => match mouse_btn {
                        MouseButton::Left => button_state_ref.store(false, Ordering::Relaxed),
                        _ => {}
                    },

                    SimulatorEvent::MouseButtonDown { mouse_btn, .. } => match mouse_btn {
                        MouseButton::Left => button_state_ref.store(true, Ordering::Relaxed),
                        _ => {}
                    },

                    SimulatorEvent::Quit => slint::quit_event_loop().unwrap(),
                    _ => {}
                }
            }
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

    // 模拟启动过程
    let u = app.get_app_window();
    if let Some(ui) = u.upgrade() {
        ui.invoke_set_boot_state(BootState::Booting);
    }
    thread::spawn(move || {
        thread::sleep(Duration::from_secs(1));
        u.upgrade_in_event_loop(|ui| {
            ui.invoke_set_boot_state(BootState::Connecting);
        })
        .unwrap();
        thread::sleep(Duration::from_secs(1));
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

    // fps计数器
    let ui = app.get_app_window();
    let frame_timer = slint::Timer::default();
    frame_timer.start(
        slint::TimerMode::Repeated,
        Duration::from_secs(1),
        move || {
            if let Some(ui) = ui.upgrade() {
                ui.set_fps(*fps.borrow());
            }
            info!("FPS: {}", *fps.borrow());
            *fps.borrow_mut() = 0;
        },
    );

    app.run().unwrap();
    Ok(())
}
