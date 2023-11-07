use std::{thread, time::Duration};

use desktop_svc::http::client::HttpClientAdapterConnection;
use embedded_graphics::prelude::*;
use embedded_graphics_simulator::{
    sdl2::{Keycode, MouseButton},
    OutputSettingsBuilder, SimulatorDisplay, SimulatorEvent, Window,
};
use embedded_svc::http::client::Client as HttpClient;
use line_buffer_provider::MyLineBufferProvider;
use slint::{
    platform::{PointerEventButton, WindowEvent},
    LogicalPosition,
};
use slint_app::MyAppDeps;
mod line_buffer_provider;
mod platform;

fn main() -> anyhow::Result<()> {
    let window = platform::MyPlatform::init()?;
    let conn = HttpClientAdapterConnection::new();
    let _app = slint_app::MyApp::new(MyAppDeps { http_conn: conn });
    let mut display = SimulatorDisplay::new(Size::new(240, 240));
    let output_settings = OutputSettingsBuilder::new().build();
    let mut simulator_window = Window::new("Desktop Simulator", &output_settings);

    let w = _app.get_app_window_as_weak();
    thread::spawn(move || {
        thread::sleep(Duration::from_secs(3));
        w.upgrade().unwrap().set_page_id(0);
    });
    loop {
        slint::platform::update_timers_and_animations();
        let redraw = window.draw_if_needed(|renderer| {
            let provider = MyLineBufferProvider::new(&mut display);
            renderer.render_by_line(provider);
        });
        if redraw {
            simulator_window.update(&display);
        }
        if window.has_active_animations() {
            continue;
        }
        for event in simulator_window.events() {
            match event {
                SimulatorEvent::KeyDown {
                    keycode,
                    keymod,
                    repeat,
                } => match keycode {
                    Keycode::Up => {
                        _app.go_to_prev_page();
                    }
                    Keycode::Down => {
                        _app.go_to_next_page();
                    }
                    _ => {}
                },
                SimulatorEvent::Quit => return Ok(()),
                SimulatorEvent::MouseButtonUp { mouse_btn, point } => match mouse_btn {
                    MouseButton::Left => {
                        window.dispatch_event(WindowEvent::PointerReleased {
                            position: LogicalPosition::new(point.x as _, point.y as _),
                            button: PointerEventButton::Left,
                        });
                    }
                    _ => {}
                },
                SimulatorEvent::MouseButtonDown { mouse_btn, point } => match mouse_btn {
                    MouseButton::Left => {
                        window.dispatch_event(WindowEvent::PointerPressed {
                            position: LogicalPosition::new(point.x as _, point.y as _),
                            button: PointerEventButton::Left,
                        });
                    }
                    _ => {}
                },
                _ => {}
            }
        }
    }
}
