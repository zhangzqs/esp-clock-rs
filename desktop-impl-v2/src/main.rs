use std::{cell::RefCell, rc::Rc, time::Duration};

use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::{
    geometry::Size,
    pixelcolor::{Rgb888, RgbColor},
};
use embedded_graphics_simulator::{
    sdl2::{Keycode, MouseButton},
    OutputSettingsBuilder, SimulatorDisplay, SimulatorEvent, Window,
};
use embedded_software_slint_backend::MySoftwarePlatform;
use log::info;
use slint::{
    platform::{Platform, PointerEventButton, WindowEvent},
    ComponentHandle, LogicalPosition, PlatformError,
};
use slint_app_v2::{get_app_window, get_schedular};

fn main() {
    std::env::set_var("RUST_LOG", "debug");
    env_logger::init();
    info!("Starting desktop simulator");
    let display = Rc::new(RefCell::new(SimulatorDisplay::<Rgb888>::new(Size::new(
        240, 240,
    ))));

    let output_settings = OutputSettingsBuilder::new().build();
    let sdl_window = Rc::new(RefCell::new(Window::new(
        "Desktop Simulator",
        &output_settings,
    )));
    info!("window has been created");

    let platform = MySoftwarePlatform::new(
        display.clone(),
        Some({
            let display_ref = display.clone();
            let window_ref = sdl_window.clone();
            move |_| -> Result<(), PlatformError> {
                window_ref.borrow_mut().update(&display_ref.borrow());
                return Ok(());
            }
        }),
    );
    let slint_window = platform.get_software_window();
    slint::platform::set_platform(Box::new(platform)).unwrap();

    let event_timer = slint::Timer::default();
    event_timer.start(slint::TimerMode::Repeated, Duration::from_millis(20), {
        let window_ref = sdl_window.clone();
        move || {
            for event in window_ref.borrow_mut().events() {
                match event {
                    SimulatorEvent::MouseButtonUp { mouse_btn, point } => match mouse_btn {
                        MouseButton::Left => {
                            slint_window.dispatch_event(WindowEvent::PointerReleased {
                                position: LogicalPosition::new(point.x as _, point.y as _),
                                button: PointerEventButton::Left,
                            });
                        }
                        _ => {}
                    },

                    SimulatorEvent::MouseButtonDown { mouse_btn, point } => match mouse_btn {
                        MouseButton::Left => {
                            slint_window.dispatch_event(WindowEvent::PointerPressed {
                                position: LogicalPosition::new(point.x as _, point.y as _),
                                button: PointerEventButton::Left,
                            });
                        }
                        _ => {}
                    },

                    SimulatorEvent::Quit => slint::quit_event_loop().unwrap(),
                    _ => {}
                }
            }
        }
    });

    let mut sche = get_schedular();
    let sche_timer = slint::Timer::default();
    sche_timer.start(
        slint::TimerMode::Repeated,
        Duration::from_millis(20),
        move || {
            sche.schedule_once();
        },
    );

    slint::run_event_loop_until_quit().unwrap();
}

fn main1() {
    let app = get_app_window();
    let mut sche = get_schedular();
    let sche_timer = slint::Timer::default();
    sche_timer.start(
        slint::TimerMode::Repeated,
        Duration::from_millis(20),
        move || {
            sche.schedule_once();
        },
    );

    if let Some(x) = app.upgrade() {
        x.run().unwrap();
    }
}