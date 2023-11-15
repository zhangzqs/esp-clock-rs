use std::{thread, time::Duration, sync::Arc, rc::Rc, borrow::BorrowMut, cell::RefCell};

use desktop_svc::http::client::HttpClientAdapterConnection;
use embedded_graphics::prelude::*;
use embedded_graphics_simulator::{
    sdl2::{Keycode, MouseButton},
    OutputSettingsBuilder, SimulatorDisplay, SimulatorEvent, Window,
};
use embedded_svc::http::client::Client as HttpClient;
use line_buffer_provider::MyLineBufferProvider;
use log::info;
use slint::{
    platform::{PointerEventButton, WindowEvent},
    LogicalPosition,
};
use slint_app::MyAppDeps;

use button_driver::{Button, ButtonConfig, PinWrapper};
mod line_buffer_provider;
mod platform;

#[derive(Clone)]
struct MyButtonPin(Rc<RefCell<bool>>);

impl PinWrapper for MyButtonPin {
    fn is_high(&self) -> bool {
        *self.0.borrow()
    }
}

fn main() -> anyhow::Result<()> {
    env_logger::init();
    info!("Starting desktop simulator");
    let window = platform::MyPlatform::init()?;
    let conn = HttpClientAdapterConnection::new();
    let _app = slint_app::MyApp::new(MyAppDeps { http_conn: conn });
    let mut display = SimulatorDisplay::new(Size::new(240, 240));
    let output_settings = OutputSettingsBuilder::new().build();
    let mut simulator_window = Window::new("Desktop Simulator", &output_settings);
    
    let u = _app.get_app_window_as_weak();
    thread::spawn(move || {
        thread::sleep(Duration::from_secs(1));
        if let Some(ui) = u.upgrade() {
            ui.invoke_boot();
        }
        thread::sleep(Duration::from_secs(3));
        if let Some(ui) = u.upgrade() {
            ui.invoke_goto_home();
        }
    });

    // 假设代表按键状态，默认为松开，值为false
    let br = Rc::new(RefCell::new(false));
    let mut button = Button::new(MyButtonPin(br.clone()), ButtonConfig{
        mode: button_driver::Mode::PullDown, // 当按键松开时，是低电平
        ..Default::default()
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
                SimulatorEvent::KeyUp { keycode,.. }=>match keycode {
                    Keycode::Space => {
                        br.clone().borrow_mut().replace(false);
                    }
                    _=>{}
                }
                SimulatorEvent::KeyDown {
                    keycode,..
                } => match keycode {
                    Keycode::Space => {
                        br.clone().borrow_mut().replace(true);
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
        button.tick();
        if button.is_clicked() {
            info!("Click");
            _app.on_one_button_click();
        } else if button.is_double_clicked() {
            info!("Double click");
            _app.on_one_button_double_click();
        } else if button.is_triple_clicked() {
            info!("Triple click");
        } else if let Some(dur) = button.current_holding_time() {
            info!("Held for {dur:?}");
        } else if let Some(dur) = button.held_time() {
            info!("Total holding time {dur:?}");
        }
        button.reset();
    }
}
