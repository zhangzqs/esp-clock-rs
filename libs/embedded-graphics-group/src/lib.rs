use std::{usize, cell::RefCell, rc::Rc};

use embedded_graphics::{
    draw_target::DrawTarget, geometry::OriginDimensions, pixelcolor::PixelColor, primitives::{Rectangle, PointsIter},
};
use log::info;

type ActivateCallback = Box<dyn FnMut(usize, usize)>;

pub struct LogicalDisplay<C, D>
where
    C: PixelColor,
    D: DrawTarget<Color = C>,
{
    parent: Rc<RefCell<DisplayGroup<C, D>>>,
    aria: Rectangle,
    is_active: bool,
    on_activate: Option<ActivateCallback>,
}

impl<C, D> LogicalDisplay<C, D>
where
    C: PixelColor,
    D: DrawTarget<Color = C>,
{
    pub fn new(parent: Rc<RefCell<DisplayGroup<C, D>>>, aria: Rectangle) -> Rc<RefCell<Self>> {
        let child = Rc::new(RefCell::new(Self {
            parent:parent.clone(),
            aria,
            is_active: false,
            on_activate: None,
        }));
        parent.borrow_mut().logical_displays.push(child.clone());
        child
    }
}

impl<C, D> OriginDimensions for LogicalDisplay<C, D>
where
    C: PixelColor,
    D: DrawTarget<Color = C>,
{
    fn size(&self) -> embedded_graphics::geometry::Size {
        self.aria.size
    }
}

impl<C, D> DrawTarget for LogicalDisplay<C, D>
where
    C: PixelColor,
    D: DrawTarget<Color = C>,
{
    type Color = C;

    type Error = D::Error;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = embedded_graphics::prelude::Pixel<Self::Color>>,
    {
        if !self.is_active {
            return Ok(());
        }

        let parent_ref = self.parent.clone();
        let parent = parent_ref.borrow();
        let mut phy_display = parent.physical_display.borrow_mut();

        let origin = self.aria.top_left;
        
        phy_display.draw_iter(pixels.into_iter().map(|mut p| {
            p.0 += origin;
            p
        }))
    }
}

pub struct DisplayGroup<C, D>
where
    C: PixelColor,
    D: DrawTarget<Color = C>,
{
    physical_display: Rc<RefCell<D>>,
    logical_displays: Vec<Rc<RefCell<LogicalDisplay<C, D>>>>,
    current_active_display: isize,
}

impl<C, D> DisplayGroup<C, D>
where
    C: PixelColor,
    D: DrawTarget<Color = C>,
{
    pub fn new(physical_display: Rc<RefCell<D>>, initial_logical_display_capacity: usize) -> Self {
        Self {
            physical_display,
            logical_displays: Vec::with_capacity(initial_logical_display_capacity),
            current_active_display: -1,
        }
    }

    pub fn get_current_active_display_index(&self) -> isize {
        self.current_active_display
    }

    pub fn switch_to_logical_display(&mut self, index: isize) -> Rc<RefCell<LogicalDisplay<C, D>>> {
        if index < 0 || index >= self.logical_displays.len() as isize {
            panic!("index out of range");
        }
        if self.current_active_display != -1 {
            let old_display_ref = &self.logical_displays[self.current_active_display as usize];
            let mut old_display = old_display_ref.borrow_mut();
            old_display.is_active = false;
        }

        self.current_active_display = index;
        let new_display_ref = &self.logical_displays[self.current_active_display as usize];
        let mut new_display = new_display_ref.borrow_mut();
        new_display.is_active = true;
        info!("switch to logical display {}", index);
        new_display_ref.clone()
    }
}
