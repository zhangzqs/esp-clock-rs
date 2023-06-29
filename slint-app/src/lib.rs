#![no_std]
slint::include_modules!();

pub fn create_app() -> AppWindow {
    let ui = AppWindow::new().expect("Failed to create AppWindow");
    let _ = ui.as_weak();
    ui
}