use embedded_graphics::prelude::*;
use embedded_graphics_simulator::{SimulatorDisplay, Window, OutputSettingsBuilder, SimulatorEvent, sdl2::MouseButton};
use line_buffer_provider::MyLineBufferProvider;
use slint::{platform::{WindowEvent, PointerEventButton}, LogicalPosition};

mod platform;
mod line_buffer_provider;

fn main() -> anyhow::Result<()>{
    let window = platform::MyPlatform::init()?;
    let _ui = slint_app::create_app();
    let mut display = SimulatorDisplay::new(Size::new(240, 240));
    let output_settings = OutputSettingsBuilder::new().build();
    let mut simulator_window = Window::new("Desktop Simulator", &output_settings);
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
                SimulatorEvent::Quit => return Ok(()),
                SimulatorEvent::MouseButtonUp { mouse_btn, point } =>{
                    match mouse_btn {
                        MouseButton::Left => {
                            window.dispatch_event(WindowEvent::PointerReleased { 
                                position: LogicalPosition::new(point.x as _, point.y as _), 
                                button: PointerEventButton::Left, 
                            });
                        }
                        _ => {}
                    }
                }
                SimulatorEvent::MouseButtonDown { mouse_btn, point } =>{
                    match mouse_btn {
                        MouseButton::Left => {
                            window.dispatch_event(WindowEvent::PointerPressed { 
                                position: LogicalPosition::new(point.x as _, point.y as _), 
                                button: PointerEventButton::Left, 
                            });
                        }
                        _ => {}
                    }
                }
                _ => {}
            }
        }
    }
}
