use std::{
    cell::RefCell,
    rc::Rc,
    sync::{Arc, Mutex},
    thread,
};

use log::{error, info};
use slint::{
    platform::{
        software_renderer::{LineBufferProvider, MinimalSoftwareWindow, RepaintBufferType},
        EventLoopProxy, Platform, WindowAdapter,
    },
    EventLoopError, PlatformError, Rgb8Pixel,
};

use embedded_graphics::{pixelcolor::Rgb888, prelude::*, primitives::Rectangle};

struct MyLineBufferProvider<'a, DrawTarget> {
    display: &'a mut DrawTarget,
    line_buffer: &'a mut [Rgb8Pixel],
}

impl<DrawTarget, EmbeddedPixelColor> LineBufferProvider for MyLineBufferProvider<'_, DrawTarget>
where
    DrawTarget: embedded_graphics::draw_target::DrawTarget<Color = EmbeddedPixelColor>,
    EmbeddedPixelColor: From<Rgb888>,
{
    type TargetPixel = Rgb8Pixel;

    fn process_line(
        &mut self,
        line: usize,
        range: core::ops::Range<usize>,
        render_fn: impl FnOnce(&mut [Self::TargetPixel]),
    ) {
        let rect = Rectangle {
            top_left: Point::new(range.start as _, line as _),
            size: Size::new(range.len() as _, 1),
        };
        render_fn(&mut self.line_buffer[range]);
        self.display
            .fill_contiguous(
                &rect,
                self.line_buffer
                    .iter()
                    .map(|p| Rgb888::new(p.r, p.g, p.b).into()),
            )
            .map_err(drop)
            .unwrap();
    }
}

pub struct MySoftwarePlatform<DrawTarget, Callback> {
    display: Rc<RefCell<DrawTarget>>,
    window: Rc<MinimalSoftwareWindow>,
    start_time: std::time::Instant,
    event_loop_queue: Arc<Mutex<Vec<EventQueueElement>>>,
    redraw_callback: Option<Rc<RefCell<Callback>>>,
}

impl<DrawTarget, Callback> MySoftwarePlatform<DrawTarget, Callback> {
    pub fn new(display: Rc<RefCell<DrawTarget>>, redraw_callback: Option<Callback>) -> Self {
        Self {
            display,
            window: MinimalSoftwareWindow::new(Default::default()),
            redraw_callback: redraw_callback.map(|x| Rc::new(RefCell::new(x))),
            start_time: std::time::Instant::now(),
            event_loop_queue: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub fn get_software_window(&self) -> Rc<MinimalSoftwareWindow> {
        self.window.clone()
    }
}

impl<DrawTarget, EmbeddedPixelColor, Callback> Platform for MySoftwarePlatform<DrawTarget, Callback>
where
    DrawTarget: embedded_graphics::draw_target::DrawTarget<Color = EmbeddedPixelColor>,
    EmbeddedPixelColor: From<Rgb888>,
    Callback: FnMut(bool) -> Result<(), PlatformError> + 'static,
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
        let window = self.window.clone();
        let size = self.display.borrow().bounding_box().size;
        let mut line_buffer = vec![Rgb8Pixel::default(); size.width as usize];
        window.set_size(slint::PhysicalSize::new(size.width, size.height));

        loop {
            if let Some(d) = slint::platform::duration_until_next_timer_update() {
                thread::sleep(d);
            }
            slint::platform::update_timers_and_animations();
            let redraw = window.draw_if_needed(|renderer| {
                renderer.render_by_line(MyLineBufferProvider {
                    display: &mut (*self.display.borrow_mut()),
                    line_buffer: &mut line_buffer,
                });
            });

            if redraw {
                if let Some(f) = self.redraw_callback.clone() {
                    if let Err(e) = f.borrow_mut()(redraw) {
                        error!("Error in event loop callback: {:?}", e);
                        return Err(e);
                    }
                }
            }

            // 动画没处理完优先处理动画
            if window.has_active_animations() {
                continue;
            }

            // process event in event loop from event queue
            let mut queue = self.event_loop_queue.lock().unwrap();
            for event in queue.drain(..) {
                match event {
                    EventQueueElement::Quit => {
                        return Ok(());
                    }
                    EventQueueElement::Invoke(f) => f(),
                }
            }
        }
    }
}

enum EventQueueElement {
    Quit,
    Invoke(Box<dyn FnOnce() + Send>),
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
