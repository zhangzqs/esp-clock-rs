use std::{ops::Range, rc::Rc, time::Duration};

use embedded_graphics::{
    pixelcolor::{raw::RawU16, Rgb565},
    prelude::{DrawTarget, Point, Size},
    primitives::Rectangle,
};
use esp_idf_svc::systime::EspSystemTime;
use slint::{
    platform::{
        software_renderer::{LineBufferProvider, MinimalSoftwareWindow, Rgb565Pixel},
        Platform, WindowAdapter,
    },
    PlatformError,
};

pub struct MyPlatform {
    pub window: Rc<MinimalSoftwareWindow>,
}

impl Platform for MyPlatform {
    fn create_window_adapter(&self) -> Result<Rc<dyn WindowAdapter>, PlatformError> {
        Ok(self.window.clone())
    }

    fn duration_since_start(&self) -> Duration {
        EspSystemTime {}.now()
    }
}
pub struct MyLineBufferProvider<'a, T>
where
    T: DrawTarget<Color = Rgb565>,
{
    pub display: &'a mut T,
    pub line_buffer: &'a mut [Rgb565Pixel],
}

impl<T: DrawTarget<Color = Rgb565>> LineBufferProvider for MyLineBufferProvider<'_, T> {
    type TargetPixel = Rgb565Pixel;

    fn process_line(
        &mut self,
        line: usize,
        range: Range<usize>,
        render_fn: impl FnOnce(&mut [Self::TargetPixel]),
    ) {
        let rect = Rectangle::new(
            Point::new(range.start as _, line as _),
            Size::new(range.len() as _, 1),
        );
        render_fn(&mut self.line_buffer[range]);
        self.display
            .fill_contiguous(
                &rect,
                self.line_buffer
                    .iter()
                    .map(|p| Rgb565::from(RawU16::from(p.0))),
            )
            .map_err(drop)
            .unwrap();
    }
}
