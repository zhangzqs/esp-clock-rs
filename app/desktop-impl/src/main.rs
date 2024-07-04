use app_core::{get_app_window, get_scheduler, Scheduler};
use log::info;
use std::rc::Rc;
use std::time::Duration;

mod http_client;
use http_client::HttpClient;

mod http_server;
use http_server::HttpServer;

mod midi_player;
use midi_player::MidiPlayer;

fn start_scheduler() -> Rc<Scheduler> {
    let sche = get_scheduler();
    sche.register_node(HttpClient::new(4));
    sche.register_node(HttpServer::new());
    sche.register_node(MidiPlayer::new());
    sche
}

fn common_init() {
    std::env::set_var("RUST_LOG", "debug");
    env_logger::init();
}

fn _hardware_main() {
    use slint::ComponentHandle;

    common_init();
    info!("hardware renderer mode");
    let app = get_app_window();
    let sche = start_scheduler();
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

fn _software_main() {
    use std::cell::RefCell;

    use embedded_graphics::{geometry::Size, pixelcolor::Rgb888};
    use embedded_graphics_simulator::{
        sdl2::MouseButton, OutputSettingsBuilder, SimulatorDisplay, SimulatorEvent, Window,
    };
    use embedded_software_slint_backend::MySoftwarePlatform;
    use slint::{
        platform::{PointerEventButton, WindowEvent},
        LogicalPosition, PlatformError,
    };

    common_init();
    log::info!("software renderer mode");

    let display = Rc::new(RefCell::new(SimulatorDisplay::<Rgb888>::new(Size::new(
        240, 240,
    ))));

    let output_settings = OutputSettingsBuilder::new().build();
    let sdl_window = Rc::new(RefCell::new(Window::new(
        "Desktop Simulator",
        &output_settings,
    )));
    info!("window has been created");

    let first_frame_has_render = Rc::new(RefCell::new(false));
    let platform = MySoftwarePlatform::new(
        display.clone(),
        Some({
            let display_ref = display.clone();
            let window_ref = sdl_window.clone();
            let first_frame_has_render_ref = first_frame_has_render.clone();
            move |_| -> Result<(), PlatformError> {
                window_ref.borrow_mut().update(&display_ref.borrow());
                *first_frame_has_render_ref.borrow_mut() = true;
                Ok(())
            }
        }),
    );
    let slint_window = platform.get_software_window();
    slint::platform::set_platform(Box::new(platform)).unwrap();

    let event_timer = slint::Timer::default();
    event_timer.start(slint::TimerMode::Repeated, Duration::from_millis(20), {
        let window_ref = sdl_window.clone();
        let first_frame_has_render_ref = first_frame_has_render.clone();
        move || {
            // 第一帧还没渲染好，不处理事件
            if !*first_frame_has_render_ref.borrow() {
                return;
            }
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

    let sche = start_scheduler();
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

fn main() {
    #[cfg(not(feature = "software-renderer"))]
    _hardware_main();
    #[cfg(feature = "software-renderer")]
    _software_main();
}
