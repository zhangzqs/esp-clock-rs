use std::sync::{Arc, Mutex};

use embedded_graphics::pixelcolor::RgbColor;

pub struct SlintPixelBufferDrawTarget {
    pub buf: Arc<Mutex<slint::SharedPixelBuffer<slint::Rgb8Pixel>>>,
}

impl SlintPixelBufferDrawTarget {
    pub fn new(buf: Arc<Mutex<slint::SharedPixelBuffer<slint::Rgb8Pixel>>>) -> Self {
        Self { buf }
    }
}

impl embedded_graphics::geometry::OriginDimensions for SlintPixelBufferDrawTarget {
    fn size(&self) -> embedded_graphics::prelude::Size {
        let buf = self.buf.lock().unwrap();
        embedded_graphics::prelude::Size::new(buf.width() as u32, buf.height() as u32)
    }
}

impl embedded_graphics::draw_target::DrawTarget for SlintPixelBufferDrawTarget {
    type Color = embedded_graphics::pixelcolor::Rgb888;

    type Error = ();

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = embedded_graphics::prelude::Pixel<Self::Color>>,
    {
        let mut buf = self.buf.lock().unwrap();
        let width = buf.width();
        let bs = buf.make_mut_bytes();
        for pixel in pixels {
            let p = pixel.0;
            let rgb = pixel.1;
            let (r, g, b) = (rgb.r(), rgb.g(), rgb.b());
            let idx = (p.y * width as i32 + p.x) as usize;
            bs[idx * 3 + 0] = r;
            bs[idx * 3 + 1] = g;
            bs[idx * 3 + 2] = b;
        }
        Ok(())
    }
}
