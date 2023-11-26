use embedded_graphics::{
    draw_target::DrawTarget,
    framebuffer::Framebuffer,
    mock_display::MockDisplay,
    pixelcolor::{raw::{LittleEndian, BigEndian}, Rgb888}, primitives::Rectangle, prelude::*,
};
use log::info;
use slint::{Image, Rgb8Pixel, SharedPixelBuffer};
use std::{
    thread, rc::Rc, sync::{Arc, Mutex}, cell::RefCell,
    time::{Duration, Instant},
};
use embedded_graphics_group::{LogicalDisplay, DisplayGroup};
use wasm_log;
use embedded_software_slint_backend::EmbeddedSoftwarePlatform;
use embedded_software_slint_backend::RGB888PixelColorAdapter;

#[cfg_attr(target_arch = "wasm32", wasm_bindgen::prelude::wasm_bindgen(start))]
pub fn main() {
    wasm_log::init(wasm_log::Config::default());
    info!("Log init");


    let mut physical_display = Arc::new(Mutex::new(Framebuffer::<Rgb888, _, BigEndian, 240, 240, 172800>::new()));

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

    
}