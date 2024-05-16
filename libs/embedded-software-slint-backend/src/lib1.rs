use core::{cell::RefCell, ops::Range};

use std::{
    rc::Rc,
    sync::{Arc, Mutex},
    thread,
};

use log::{error, info};

use slint::{
    platform::{
        software_renderer::{LineBufferProvider, MinimalSoftwareWindow, TargetPixel as SlintPixel},
        EventLoopProxy, Platform, WindowAdapter,
    },
    EventLoopError, PlatformError,
};

use embedded_graphics::{
    pixelcolor::{raw::RawU16, PixelColor as EmbeddedPixelColor},
    prelude::*,
    primitives::Rectangle,
};

pub trait PixelColorAdapter<EC, STC>: Default
where
    EC: EmbeddedPixelColor,
    STC: SlintPixel + Default,
{
    fn convert(self, pixel: &STC) -> EC;
}

#[derive(Default)]
pub struct RGB888PixelColorAdapter;

impl PixelColorAdapter<embedded_graphics::pixelcolor::Rgb888, slint::Rgb8Pixel>
    for RGB888PixelColorAdapter
{
    fn convert(self, pixel: &slint::Rgb8Pixel) -> embedded_graphics::pixelcolor::Rgb888 {
        embedded_graphics::pixelcolor::Rgb888::new(pixel.r, pixel.g, pixel.b)
    }
}

#[derive(Default)]
pub struct RGB565PixelColorAdapter;

impl
    PixelColorAdapter<
        embedded_graphics::pixelcolor::Rgb565,
        slint::platform::software_renderer::Rgb565Pixel,
    > for RGB565PixelColorAdapter
{
    fn convert(
        self,
        pixel: &slint::platform::software_renderer::Rgb565Pixel,
    ) -> embedded_graphics::pixelcolor::Rgb565 {
        embedded_graphics::pixelcolor::Rgb565::from(RawU16::from(pixel.0))
    }
}

struct MyLineBufferProvider<'a, T, EC, STC, PCA>
where
    T: DrawTarget<Color = EC>,
    EC: EmbeddedPixelColor,
    STC: SlintPixel + Default,
{
    display: &'a mut T,
    line_buffer: Vec<STC>,
    _phantom: std::marker::PhantomData<PCA>,
}

impl<'a, T, EC, STC, PCA> MyLineBufferProvider<'a, T, EC, STC, PCA>
where
    T: DrawTarget<Color = EC>,
    EC: EmbeddedPixelColor,
    STC: SlintPixel + Default,
    PCA: PixelColorAdapter<EC, STC>,
{
    pub fn new(display: &'a mut T) -> Self {
        let width = display.bounding_box().size.width as usize;
        Self {
            display,
            line_buffer: vec![Default::default(); width],
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<T, EC, STC, PCA> LineBufferProvider for MyLineBufferProvider<'_, T, EC, STC, PCA>
where
    T: DrawTarget<Color = EC>,
    EC: EmbeddedPixelColor,
    STC: SlintPixel + Default,
    PCA: PixelColorAdapter<EC, STC>,
{
    type TargetPixel = STC;

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
                &Rectangle {
                    top_left: Point::new(range.start as _, line as _),
                    size: Size::new(range.len() as _, 1),
                },
                self.line_buffer
                    .iter()
                    .map(|p: &STC| PCA::default().convert(p)),
            )
            .map_err(drop)
            .unwrap();
    }
}

enum EventQueueElement {
    Quit,
    Invoke(Box<dyn FnOnce() + Send>),
}

pub struct EmbeddedSoftwarePlatform<T, F, EC, STC, PCA = RGB888PixelColorAdapter>
where
    T: DrawTarget<Color = EC>,
    F: FnMut(bool) -> Result<(), PlatformError> + 'static,
    EC: EmbeddedPixelColor,
    STC: SlintPixel + Default,
    PCA: PixelColorAdapter<EC, STC>,
{
    display: Arc<Mutex<T>>,
    window: Rc<MinimalSoftwareWindow>,
    start_time: std::time::Instant,
    event_loop_callback: Option<Rc<RefCell<F>>>,
    event_loop_queue: Arc<Mutex<Vec<EventQueueElement>>>,
    _phantom: std::marker::PhantomData<(EC, STC, PCA)>,
}

impl<T, F, EC, STC, PCA> EmbeddedSoftwarePlatform<T, F, EC, STC, PCA>
where
    T: DrawTarget<Color = EC>,
    F: FnMut(bool) -> Result<(), PlatformError> + 'static,
    EC: EmbeddedPixelColor,
    STC: SlintPixel + Default,
    PCA: PixelColorAdapter<EC, STC>,
{
    pub fn new(
        display: Arc<Mutex<T>>,
        event_loop_callback: Option<F>,
    ) -> EmbeddedSoftwarePlatform<T, F, EC, STC, PCA> {
        let window = MinimalSoftwareWindow::new(Default::default());
        EmbeddedSoftwarePlatform {
            window: window.clone(),
            start_time: std::time::Instant::now(),
            event_loop_callback: event_loop_callback.map(|f| Rc::new(RefCell::new(f))),
            event_loop_queue: Arc::new(Mutex::new(Vec::new())),
            _phantom: std::marker::PhantomData,
            display,
        }
    }
}

impl<T, F, EC, STC, PCA> Platform for EmbeddedSoftwarePlatform<T, F, EC, STC, PCA>
where
    T: DrawTarget<Color = EC>,
    F: FnMut(bool) -> Result<(), PlatformError> + 'static,
    EC: EmbeddedPixelColor,
    STC: SlintPixel + Default,
    PCA: PixelColorAdapter<EC, STC>,
{
    fn create_window_adapter(&self) -> Result<Rc<dyn WindowAdapter>, PlatformError> {
        Ok(self.window.clone())
    }

    fn duration_since_start(&self) -> core::time::Duration {
        self.start_time.elapsed()
    }

    fn new_event_loop_proxy(&self) -> Option<Box<dyn EventLoopProxy>> {
        Some(Box::new(MyEventLoopProxy {
            queue: self.event_loop_queue.clone(),
        }))
    }

    fn run_event_loop(&self) -> Result<(), PlatformError> {
        info!("Starting event loop");
        let window = self.window.clone();
        loop {
            // render
            if let Some(d) = slint::platform::duration_until_next_timer_update() {
                thread::sleep(d);
            }
            slint::platform::update_timers_and_animations();
            let redraw = {
                let mut d = self.display.lock().unwrap();
                window.draw_if_needed(|renderer| {
                    let provider = MyLineBufferProvider::<T, EC, STC, PCA>::new(&mut *d);
                    renderer.render_by_line(provider);
                })
            };
            if let Some(f) = self.event_loop_callback.clone() {
                if let Err(e) = f.borrow_mut()(redraw) {
                    error!("Error in event loop callback: {:?}", e);
                    return Err(e);
                }
            }
            if window.has_active_animations() {
                continue;
            }

            // process event in event loop from event queue
            let mut queue = self.event_loop_queue.lock().unwrap();
            for event in queue.drain(..) {
                match event {
                    EventQueueElement::Quit => {
                        info!("Quit event loop");
                        return Ok(());
                    }
                    EventQueueElement::Invoke(f) => f(),
                }
            }
        }
    }
}

struct MyEventLoopProxy {
    pub queue: Arc<Mutex<Vec<EventQueueElement>>>,
}

impl EventLoopProxy for MyEventLoopProxy {
    fn quit_event_loop(&self) -> Result<(), EventLoopError> {
        self.queue.lock().unwrap().push(EventQueueElement::Quit);
        Ok(())
    }

    fn invoke_from_event_loop(
        &self,
        event: Box<dyn FnOnce() + Send>,
    ) -> Result<(), EventLoopError> {
        self.queue
            .lock()
            .unwrap()
            .push(EventQueueElement::Invoke(event));
        Ok(())
    }
}
