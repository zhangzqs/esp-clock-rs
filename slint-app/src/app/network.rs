

use crate::AppWindow;


use slint::{Rgb8Pixel, SharedPixelBuffer, Weak};

pub struct NetworkMonitorApp {
    app: Weak<AppWindow>,
    image: SharedPixelBuffer<Rgb8Pixel>,
}

impl NetworkMonitorApp {
    pub fn new(app: Weak<AppWindow>) -> Self {
        let image = SharedPixelBuffer::new(120, 120);
        let _image_ref = image.clone();
        let _app_ref = app.clone();
        // slint::Timer::single_shot(Duration::from_secs(3), move || {
        //     info!("timer start");
        //     let mut e = embedded_graphics_slint_image_buf::SlintPixelBufferDrawTarget {
        //         buf: image_ref.clone(),
        //     };
        //     e.clear(Rgb888::RED).unwrap();
        //     if let Some(ui) = app_ref.upgrade() {
        //         ui.set_network_monitor_page_plot(Image::from_rgb8(image_ref.clone()));
        //     }
        // });
        Self { app, image }
    }
}
