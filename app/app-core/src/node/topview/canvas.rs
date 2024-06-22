use proto::*;
use slint::{ComponentHandle, Image, Rgb8Pixel, SharedPixelBuffer};
use std::{cell::RefCell, rc::Rc};

use crate::{get_app_window, ui};
pub struct CanvasView {
    spb: RefCell<Option<SharedPixelBuffer<Rgb8Pixel>>>,
}

impl CanvasView {
    pub fn new() -> Self {
        Self {
            spb: Default::default(),
        }
    }

    fn show(&self) {
        if let Some(ui) = get_app_window().upgrade() {
            let vm = ui.global::<ui::CanvasViewModel>();
            vm.set_show(true);
        }
    }

    fn close(&self) {
        if let Some(ui) = get_app_window().upgrade() {
            let vm = ui.global::<ui::CanvasViewModel>();
            vm.set_show(false);
        }
    }

    fn update_frame(&self) {
        if let Some(ui) = get_app_window().upgrade() {
            let vm = ui.global::<ui::CanvasViewModel>();
            vm.set_image(Image::from_rgb8(
                self.spb.borrow().as_ref().unwrap().clone(),
            ))
        }
    }
}

impl Node for CanvasView {
    fn node_name(&self) -> NodeName {
        NodeName::Canvas
    }

    fn handle_message(&self, _ctx: Rc<dyn Context>, msg: MessageWithHeader) -> HandleResult {
        match msg.body {
            Message::Lifecycle(LifecycleMessage::Init) => {
                *self.spb.borrow_mut() = Some(SharedPixelBuffer::new(240, 240));
            }
            Message::Canvas(cm) => match cm {
                CanvasMessage::Open => {
                    self.show();
                }
                CanvasMessage::Close => {
                    self.close();
                }
                CanvasMessage::Clear((r, g, b)) => {
                    if let Some(x) = self.spb.borrow_mut().as_mut() {
                        x.make_mut_slice().iter_mut().for_each(|x| {
                            (x.r, x.g, x.b) = (r, g, b);
                        });
                    }
                    self.update_frame();
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
