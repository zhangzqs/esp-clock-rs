use std::{cell::RefCell, rc::Rc, time::Duration, env::set_var, thread};

use anyhow::anyhow;
use desktop_svc::http::client::HttpClientAdapterConnection;
use embedded_graphics::prelude::*;
use embedded_graphics_simulator::{
    sdl2::Keycode, OutputSettingsBuilder, SimulatorDisplay, SimulatorEvent, Window,
};

use log::info;

use slint_app::{MyApp, MyAppDeps, BootState};

use button_driver::{Button, ButtonConfig, PinWrapper};
use embedded_software_slint_backend::EmbeddedSoftwarePlatform;

#[derive(Clone)]
struct MyButtonPin(Rc<RefCell<bool>>);

impl PinWrapper for MyButtonPin {
    fn is_high(&self) -> bool {
        *self.0.borrow()
    }
}

fn main() -> anyhow::Result<()> {
    set_var("RUST_LOG", "debug");
    env_logger::init();
    info!("Starting desktop simulator");

    let display = Rc::new(RefCell::new(SimulatorDisplay::new(Size::new(240, 240))));
    let output_settings = OutputSettingsBuilder::new().build();
    let window = Rc::new(RefCell::new(Window::new(
        "Desktop Simulator",
        &output_settings,
    )));

    let button_state = Rc::new(RefCell::new(false));
    let fps = Rc::new(RefCell::new(0));
    {
        let window_update_ref = window.clone();
        let display_update_ref = display.clone();
        let button_state_ref = button_state.clone();
        let fps_ref = fps.clone();

        let platform = EmbeddedSoftwarePlatform::new(
            display,
            Some(Box::new(move |has_redraw| {
                let mut window = window_update_ref.borrow_mut();
                let display = display_update_ref.borrow();
                if has_redraw {
                    window.update(&display);
                    *fps_ref.borrow_mut() += 1;
                }
                for event in window.events() {
                    match event {
                        SimulatorEvent::KeyUp { keycode, .. } => match keycode {
                            Keycode::Space => {
                                *button_state_ref.borrow_mut() = false;
                            }
                            _ => {}
                        },
                        SimulatorEvent::KeyDown { keycode, .. } => match keycode {
                            Keycode::Space => {
                                *button_state_ref.borrow_mut() = true;
                            }
                            _ => {}
                        },
                        SimulatorEvent::Quit => slint::quit_event_loop().unwrap(),
                        _ => {}
                    }
                }

                Ok(())
            })),
        );
        slint::platform::set_platform(Box::new(platform)).unwrap();
    }

    let app = Rc::new(MyApp::new(MyAppDeps {
        http_conn: HttpClientAdapterConnection::new(),
    }));

    // 分发按键事件
    // 假设代表按键状态，默认为松开，值为false
    let mut button = Button::new(
        MyButtonPin(button_state.clone()),
        ButtonConfig {
            mode: button_driver::Mode::PullDown, // 当按键松开时，是低电平
            ..Default::default()
        },
    );
    let app_ref = app.clone();
    let button_event_timer = slint::Timer::default();
    button_event_timer.start(
        slint::TimerMode::Repeated,
        Duration::from_millis(10),
        move || {
            button.tick();
            if button.clicks() > 0 {
                info!("Clicks: {}", button.clicks());
                app_ref.on_one_button_clicks(button.clicks() as i32);
            }else if let Some(dur) = button.current_holding_time() {
                info!("Held for {dur:?}");
                app_ref.on_one_button_long_pressed_holding_time(dur);
            } else if let Some(dur) = button.held_time() {
                info!("Total holding time {dur:?}");
                app_ref.on_one_button_long_pressed_held_time(dur);
            }
            button.reset();
        },
    );

    app.set_boot_state(BootState::Booting);
    let u = app.get_app_window_as_weak();
    thread::spawn(move ||{
        thread::sleep(Duration::from_secs(1));
        u.upgrade_in_event_loop(|ui| {
            ui.invoke_set_boot_state(BootState::Connecting);
        }).unwrap();
        thread::sleep(Duration::from_secs(5));
        u.upgrade_in_event_loop(|ui| {
            ui.invoke_set_boot_state(BootState::BootSuccess);
        }).unwrap();
        thread::sleep(Duration::from_secs(1));
        u.upgrade_in_event_loop(|ui| {
            ui.invoke_set_boot_state(BootState::Finished);
        }).unwrap();
    });

    // fps计数器
    let app_ref = app.clone();
    let frame_timer = slint::Timer::default();
    frame_timer.start(
        slint::TimerMode::Repeated,
        Duration::from_secs(1),
        move || {
            app_ref.get_app_window_as_weak().upgrade().and_then(|ui| {
                ui.set_fps(*fps.borrow());
                Some(())
            });
            *fps.borrow_mut() = 0;
        },
    );

    slint::run_event_loop().map_err(|e| anyhow!("{}", e))?;
    Ok(())
}
