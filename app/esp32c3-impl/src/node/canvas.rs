use app_core::proto::*;
use embedded_graphics::{
    draw_target::DrawTarget,
    pixelcolor::{PixelColor, Rgb888},
};
use embedded_graphics_mux::{DisplayMux, LogicalDisplay};
use std::{cell::RefCell, fmt::Debug, rc::Rc};

pub struct CanvasView<D>
where
    D: DrawTarget,
{
    display_mux: Rc<RefCell<DisplayMux<D>>>,
    this_display_id: usize,
    this_display: RefCell<Option<Rc<RefCell<LogicalDisplay<D>>>>>,
    old_display_id: RefCell<Option<isize>>,
}

impl<D> CanvasView<D>
where
    D: DrawTarget,
{
    pub fn new(display_mux: Rc<RefCell<DisplayMux<D>>>) -> Self {
        let display_id = LogicalDisplay::new(display_mux.clone()).borrow().get_id();
        Self {
            this_display_id: display_id,
            display_mux,
            old_display_id: Default::default(),
            this_display: Default::default(),
        }
    }
}

impl<C, D, E> Node for CanvasView<D>
where
    C: PixelColor + From<Rgb888>,
    D: DrawTarget<Color = C, Error = E>,
    E: Debug,
{
    fn node_name(&self) -> NodeName {
        NodeName::Canvas
    }

    fn handle_message(&self, _ctx: Rc<dyn Context>, msg: MessageWithHeader) -> HandleResult {
        match msg.body {
            Message::Lifecycle(LifecycleMessage::Init) => {}
            Message::Canvas(cm) => match cm {
                CanvasMessage::Open => {
                    let old_display_id = self.display_mux.borrow().active_index();
                    *self.old_display_id.borrow_mut() = Some(old_display_id);

                    let this_display = self
                        .display_mux
                        .borrow_mut()
                        .switch_to(self.this_display_id as _);
                    *self.this_display.borrow_mut() = Some(this_display);
                    return HandleResult::Finish(Message::Empty);
                }
                CanvasMessage::Close => {
                    if let Some(x) = self.old_display_id.borrow_mut().take() {
                        self.display_mux.borrow_mut().switch_to(x);
                    }
                }
                CanvasMessage::Clear((r, g, b)) => {
                    if let Some(x) = self.this_display.borrow().as_ref() {
                        let mut display = x.borrow_mut();
                        display.clear(Rgb888::new(r, g, b)).unwrap();
                    }
                }
                CanvasMessage::DrawLine(_) => todo!(),
                CanvasMessage::DrawCircle(_) => todo!(),
                CanvasMessage::DrawRectangle(_) => todo!(),
                CanvasMessage::DrawPixels(_) => todo!(),
            },
            _ => {}
        }
        HandleResult::Discard
    }
}
