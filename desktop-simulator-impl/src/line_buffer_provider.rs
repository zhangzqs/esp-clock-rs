use std::ops::Range;

use embedded_graphics::{prelude::*, pixelcolor::{Rgb565, raw::RawU16}, primitives::Rectangle};
use slint::platform::software_renderer::{LineBufferProvider, Rgb565Pixel};

pub struct MyLineBufferProvider <'a, T>
where T: DrawTarget<Color = Rgb565>{
    display: &'a mut T,
    line_buffer: Vec<Rgb565Pixel>,
}

impl <'a, T> MyLineBufferProvider <'a, T>
where T: DrawTarget<Color = Rgb565>{
    pub fn new(display: &'a mut T) -> Self {
        let width = display.bounding_box().size.width as usize;
        Self {
            display,
            line_buffer: vec![Rgb565Pixel::default(); width],
        }
    }
}

impl <T: DrawTarget<Color = Rgb565>> LineBufferProvider for MyLineBufferProvider<'_, T> {
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
                self.line_buffer.iter()
                    .map(|p| Rgb565::from(RawU16::from(p.0)))
            )
            .map_err(drop)
            .unwrap();
    }
}