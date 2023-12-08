use std::time::Duration;

use crate::AppWindow;
use embedded_graphics::{pixelcolor::{Rgb888, RgbColor}, draw_target::DrawTarget};
use embedded_graphics_framebuf::FrameBuf;
use log::info;
use slint::{Image, Rgb8Pixel, SharedPixelBuffer, Weak};

pub struct SlintPixelBufferDrawTarget {
    pub buf: SharedPixelBuffer<Rgb8Pixel>,
}

impl embedded_graphics::geometry::OriginDimensions for SlintPixelBufferDrawTarget {
    fn size(&self) -> embedded_graphics::prelude::Size {
        embedded_graphics::prelude::Size::new(self.buf.width(), self.buf.height())
    }
}

impl embedded_graphics::draw_target::DrawTarget for SlintPixelBufferDrawTarget {
    type Color = Rgb888;

    type Error = ();

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = embedded_graphics::prelude::Pixel<Self::Color>>,
    {
        use embedded_graphics::pixelcolor::RgbColor;
        let width = self.buf.width();
        let bs = self.buf.make_mut_bytes();
        for pixel in pixels {
            let p = pixel.0;
            let rgb = pixel.1;
            let (r, g, b) = (rgb.r(), rgb.g(), rgb.b());
            let idx = (p.y * width as i32 + p.x) as usize;
            bs[idx + 0] = r;
            bs[idx + 1] = g;
            bs[idx + 2] = b;
        }
        Ok(())
    }
}

pub struct NetworkMonitorApp {
    app: Weak<AppWindow>,
    image: SharedPixelBuffer<Rgb8Pixel>,
}

impl NetworkMonitorApp {
    pub fn new(app: Weak<AppWindow>) -> Self {
        let image = SharedPixelBuffer::new(120, 120);
        let image_ref = image.clone();
        let app_ref = app.clone();
        slint::Timer::single_shot(Duration::from_secs(3), move || {
            info!("timer start");
            let mut e = SlintPixelBufferDrawTarget {
                buf: image_ref.clone(),
            };
            e.clear(Rgb888::RED).unwrap();
            if let Some(ui) = app_ref.upgrade() {
                ui.set_network_monitor_page_plot(Image::from_rgb8(image_ref.clone()));
            }
        });
        Self { app, image }
    }
}
