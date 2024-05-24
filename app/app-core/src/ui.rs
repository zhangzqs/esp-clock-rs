use slint::ComponentHandle;
use slint::Weak;

slint::include_modules!();

static mut APP: Option<AppWindow> = None;

pub fn get_app_window() -> Weak<AppWindow> {
    unsafe {
        APP.get_or_insert_with(|| AppWindow::new().unwrap())
            .as_weak()
    }
}
