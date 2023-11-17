use std::{
    cell::RefCell,
    ops::Range,
    rc::Rc,
    sync::{Arc, Mutex},
    thread,
};

use log::{error, info};

use slint::{
    platform::{
        software_renderer::{LineBufferProvider, MinimalSoftwareWindow, Rgb565Pixel},
        EventLoopProxy, Platform, WindowAdapter,
    },
    EventLoopError, PlatformError,
};

use embedded_graphics::{
    pixelcolor::{raw::RawU16, Rgb565},
    prelude::*,
    primitives::Rectangle,
};

struct MyLineBufferProvider<T>
where
    T: DrawTarget<Color = Rgb565>,
{
    display: Rc<RefCell<T>>,
    line_buffer: Vec<Rgb565Pixel>,
}

impl<T> MyLineBufferProvider<T>
where
    T: DrawTarget<Color = Rgb565>,
{
    pub fn new(display: Rc<RefCell<T>>) -> Self {
        let width = display.borrow().bounding_box().size.width as usize;
        Self {
            display,
            line_buffer: vec![Rgb565Pixel::default(); width],
        }
    }
}

impl<T: DrawTarget<Color = Rgb565>> LineBufferProvider for MyLineBufferProvider<T> {
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
            .borrow_mut()
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

enum EventQueueElement {
    Quit,
    Invoke(Box<dyn FnOnce() + Send>),
}

pub struct EmbeddedSoftwarePlatform<T, F>
where
    T: DrawTarget<Color = Rgb565>,
    F: FnMut(bool) -> Result<(), PlatformError> + 'static,
{
    display: Rc<RefCell<T>>,
    window: Rc<MinimalSoftwareWindow>,
    start_time: std::time::Instant,
    event_loop_callback: Option<Rc<RefCell<F>>>,
    event_loop_queue: Arc<Mutex<Vec<EventQueueElement>>>,
}

impl<T, F> EmbeddedSoftwarePlatform<T, F>
where
    T: DrawTarget<Color = Rgb565>,
    F: FnMut(bool) -> Result<(), PlatformError> + 'static,
{
    pub fn new(display: Rc<RefCell<T>>, event_loop_callback: Option<F>) -> EmbeddedSoftwarePlatform<T, F> {
        let window = MinimalSoftwareWindow::new(Default::default());
        EmbeddedSoftwarePlatform {
            window: window.clone(),
            start_time: std::time::Instant::now(),
            event_loop_callback: event_loop_callback.map(|f| Rc::new(RefCell::new(f))),
            event_loop_queue: Arc::new(Mutex::new(Vec::new())),
            display,
        }
    }
}

impl<T, F> Platform for EmbeddedSoftwarePlatform<T, F>
where
    T: DrawTarget<Color = Rgb565>,
    F: FnMut(bool) -> Result<(), PlatformError> + 'static,
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
            slint::platform::update_timers_and_animations();
            let redraw = window.draw_if_needed(|renderer| {
                let provider = MyLineBufferProvider::new(self.display.clone());
                renderer.render_by_line(provider);
            });
            if let Some(f) = self.event_loop_callback.clone() {
                if let Err(e) = f.borrow_mut()(redraw) {
                    error!("Error in event loop callback: {:?}", e);
                    return Err(e);
                }
            }
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
            if window.has_active_animations() {
                continue;
            }
            if let Some(d) = slint::platform::duration_until_next_timer_update() {
                thread::sleep(d);
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
