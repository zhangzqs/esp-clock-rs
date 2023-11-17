use std::{cell::RefCell, rc::Rc, time::Duration};

use anyhow::anyhow;
use desktop_svc::http::client::HttpClientAdapterConnection;
use embedded_graphics::prelude::*;
use embedded_graphics_simulator::{
    sdl2::{Keycode, MouseButton},
    OutputSettingsBuilder, SimulatorDisplay, SimulatorEvent, Window,
};
use embedded_svc::http::client::Client as HttpClient;
use log::info;
use slint::{
    platform::{PointerEventButton, WindowEvent},
    LogicalPosition,
};
use slint_app::{MyApp, MyAppDeps};

use button_driver::{Button, ButtonConfig, PinWrapper};

use crate::platform::MyPlatform;
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

    let display = Rc::new(RefCell::new(SimulatorDisplay::new(Size::new(240, 240))));
    let output_settings = OutputSettingsBuilder::new().build();
    let window = Rc::new(RefCell::new(Window::new(
        "Desktop Simulator",
        &output_settings,
    )));

    let button_state = Rc::new(RefCell::new(false));
    // 假设代表按键状态，默认为松开，值为false
    let mut button = Button::new(
        MyButtonPin(button_state.clone()),
        ButtonConfig {
            mode: button_driver::Mode::PullDown, // 当按键松开时，是低电平
            ..Default::default()
        },
    );
    let on_button_click: Rc<RefCell<Option<Box<dyn Fn()>>>> = Rc::new(RefCell::new(None));
    let on_button_double_click: Rc<RefCell<Option<Box<dyn Fn()>>>> = Rc::new(RefCell::new(None));
    {
        let window_update_ref: Rc<RefCell<Window>> = window.clone();
        let display_update_ref = display.clone();
        let on_button_click = on_button_click.clone();
        let on_button_double_click = on_button_double_click.clone();

        let platform = MyPlatform::new(
            display,
            Some(Box::new(move |has_redraw| {
                let mut window = window_update_ref.borrow_mut();
                let display = display_update_ref.borrow();
                if has_redraw {
                    window.update(&display);
                }
                for event in window.events() {
                    match event {
                        SimulatorEvent::KeyUp { keycode, .. } => match keycode {
                            Keycode::Space => {
                                *button_state.borrow_mut() = false;
                            }
                            _ => {}
                        },
                        SimulatorEvent::KeyDown { keycode, .. } => match keycode {
                            Keycode::Space => {
                                *button_state.borrow_mut() = true;
                            }
                            _ => {}
                        },
                        SimulatorEvent::Quit => slint::quit_event_loop().unwrap(),
                        _ => {}
                    }
                }
                button.tick();
                if button.is_clicked() {
                    info!("Click");
                    on_button_click.borrow_mut().as_ref().map(|f| f());
                } else if button.is_double_clicked() {
                    info!("Double click");
                    on_button_double_click.borrow_mut().as_ref().map(|f| f());
                } else if button.is_triple_clicked() {
                    info!("Triple click");
                } else if let Some(dur) = button.current_holding_time() {
                    info!("Held for {dur:?}");
                } else if let Some(dur) = button.held_time() {
                    info!("Total holding time {dur:?}");
                }
                button.reset();
                Ok(())
            })),
        );
        slint::platform::set_platform(Box::new(platform)).unwrap();
    }

    let app = Rc::new(MyApp::new(MyAppDeps {
        http_conn: HttpClientAdapterConnection::new(),
    }));

    {
        let app = app.clone();
        on_button_click
            .borrow_mut()
            .replace(Box::new(move || app.on_one_button_click()));
    }
    {
        let app = app.clone();
        on_button_double_click
            .borrow_mut()
            .replace(Box::new(move || app.on_one_button_double_click()));
    }

    slint::run_event_loop().map_err(|e| anyhow!("{}", e))?;
    Ok(())
}
