use std::{cell::RefCell, env::set_var, rc::Rc, thread, time::Duration, sync::{Arc, Mutex}};


use desktop_svc::http::client::HttpClientAdapterConnection;
use embedded_graphics::{prelude::*, pixelcolor::Rgb888, primitives::Rectangle};
use embedded_graphics_group::{DisplayGroup, LogicalDisplay};
use embedded_graphics_simulator::{
    sdl2::Keycode, OutputSettingsBuilder, SimulatorDisplay, SimulatorEvent, Window,
};

use log::info;

use slint_app::{BootState, MyApp, MyAppDeps};

use button_driver::{Button, ButtonConfig, PinWrapper};
use embedded_software_slint_backend::{EmbeddedSoftwarePlatform, RGB888PixelColorAdapter};

#[derive(Clone)]
struct MyButtonPin(Rc<RefCell<bool>>);

impl PinWrapper for MyButtonPin {
    fn is_high(&self) -> bool {
        *self.0.borrow()
    }
}

struct MockSystem;
impl slint_app::System for MockSystem {
    /// 重启
    fn restart(&self) {
    }

    /// 获取剩余可用堆内存，这可能比最大连续的可分配块的值还要大
    fn get_free_heap_size(&self) -> usize {
        0
    }

    /// 获取最大连续的可分配块
    fn get_largest_free_block(&self) -> usize {
        0
    }
}



fn main() -> anyhow::Result<()> {
    set_var("RUST_LOG", "debug");
    env_logger::init();
    info!("Starting desktop simulator");

    let display = Rc::new(RefCell::new(SimulatorDisplay::<Rgb888>::new(Size::new(240, 240))));
    let output_settings = OutputSettingsBuilder::new().build();
    let window = Rc::new(RefCell::new(Window::new(
        "Desktop Simulator",
        &output_settings,
    )));

    let display_group = Rc::new(RefCell::new(DisplayGroup::new(display.clone(), 2)));
    let slint_display = LogicalDisplay::new(display_group.clone(), Rectangle{
        top_left: Point::new(0, 0),
        size: Size::new(240, 240),
    });
    let app_display = LogicalDisplay::new(display_group.clone(), Rectangle{
        top_left: Point::new(0, 120),
        size: Size::new(120, 120),
    });
    display_group.borrow_mut().switch_to_logical_display(0);
    let button_state = Rc::new(RefCell::new(false));
    let fps = Rc::new(RefCell::new(0));
    {
        let window_update_ref = window.clone();
        let display_update_ref = display.clone();
        let button_state_ref = button_state.clone();
        let fps_ref = fps.clone();
        let slint_display_ref = slint_display.clone();

        let platform = EmbeddedSoftwarePlatform::<_, _, _, _, RGB888PixelColorAdapter>::new(
            slint_display_ref,
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

    let app = MyApp::new(MyAppDeps {
        http_conn: HttpClientAdapterConnection::new(),
        system: MockSystem,
    });

    // 分发按键事件
    // 假设代表按键状态，默认为松开，值为false
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
        Duration::from_millis(10),
        move || {
            button.tick();
            if button.clicks() > 0 {
                let clicks = button.clicks();
                info!("Clicks: {}", clicks);
                u.upgrade().map(move |ui| {
                    ui.invoke_on_one_button_clicks(clicks as i32);
                    
                });
            } else if let Some(dur) = button.current_holding_time() {
                info!("Held for {dur:?}");
                u.upgrade().map(move |ui| {
                    ui.invoke_on_one_button_long_pressed_holding(dur.as_millis() as i64);
                    
                });
            } else if let Some(dur) = button.held_time() {
                info!("Total holding time {dur:?}");
                u.upgrade().map(move |ui| {
                    ui.invoke_on_one_button_long_pressed_held(dur.as_millis() as i64);
                    
                });
            }
            button.reset();
        },
    );

    // 模拟启动过程
    let u = app.get_app_window();
    u.upgrade().map(|ui| {
        ui.invoke_set_boot_state(BootState::Booting);
        
    });
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
            ui.upgrade().map(|ui| {
                ui.set_fps(*fps.borrow());
                
            });
            info!("FPS: {}", *fps.borrow());
            *fps.borrow_mut() = 0;
        },
    );

    let display_group_ref = display_group.clone();
    let timer = slint::Timer::default();
    timer.start(slint::TimerMode::Repeated, Duration::from_secs(1), move || {
        let idx = display_group_ref.borrow().get_current_active_display_index() ^ 1;
        let display = display_group_ref.borrow_mut().switch_to_logical_display(idx);
        display.borrow_mut().clear(Rgb888::RED).unwrap();
    });

    app.run().unwrap();
    Ok(())
}
