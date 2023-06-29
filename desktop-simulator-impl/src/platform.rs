use std::{rc::Rc};

use anyhow;
use slint::{
    platform::{software_renderer::MinimalSoftwareWindow, Platform, WindowAdapter},
    PlatformError,
};

pub(crate) struct MyPlatform {
    window: Rc<MinimalSoftwareWindow>,
    start_time: std::time::Instant,
}

impl MyPlatform {
    pub(crate) fn init() -> anyhow::Result<Rc<MinimalSoftwareWindow>> {
        let window = MinimalSoftwareWindow::new(Default::default());
        let platform = MyPlatform {
            window: window.clone(),
            start_time: std::time::Instant::now(),
        };
        slint::platform::set_platform(Box::new(platform))
            .map_err(|e| anyhow::anyhow!("{:?}", e))?;
        anyhow::Ok(window.clone())
    }
}

impl Platform for MyPlatform {
    fn create_window_adapter(&self) -> Result<Rc<dyn WindowAdapter>, PlatformError> {
        Ok(self.window.clone())
    }

    fn duration_since_start(&self) -> core::time::Duration {
        self.start_time.elapsed()
    }
}
