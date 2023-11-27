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
}