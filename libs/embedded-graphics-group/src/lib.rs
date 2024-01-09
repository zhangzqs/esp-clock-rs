use std::{
    sync::{Arc, Mutex},
    usize,
};

use embedded_graphics::{
    draw_target::DrawTarget,
    geometry::{OriginDimensions, Size},
    primitives::Rectangle,
};
use log::info;

pub struct LogicalDisplay<D>
where
    D: DrawTarget,
{
    parent: Arc<Mutex<DisplayGroup<D>>>,
    aria: Rectangle,
    is_active: bool,
    id: usize,
}

impl<D> LogicalDisplay<D>
where
    D: DrawTarget,
{
    pub fn new(parent: Arc<Mutex<DisplayGroup<D>>>, aria: Rectangle) -> Arc<Mutex<Self>> {
        let mut parent_mut_ref = parent.lock().unwrap();
        let id = parent_mut_ref.logical_displays.len();
        let child = Arc::new(Mutex::new(Self {
            parent: parent.clone(),
            aria,
            is_active: false,
            id,
        }));
        info!("create logical display {}", id);
        parent_mut_ref.logical_displays.push(child.clone());
        child
    }

    pub fn get_aria(&self) -> Rectangle {
        self.aria
    }

    pub fn get_id(&self) -> usize {
        self.id
    }
}

impl<D> OriginDimensions for LogicalDisplay<D>
where
    D: DrawTarget,
{
    fn size(&self) -> embedded_graphics::geometry::Size {
        self.aria.size
    }
}

impl<D> DrawTarget for LogicalDisplay<D>
where
    D: DrawTarget,
{
    type Color = D::Color;

    type Error = D::Error;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = embedded_graphics::prelude::Pixel<Self::Color>>,
    {
        if !self.is_active {
            return Ok(());
        }

        let parent_ref = self.parent.clone();
        let parent = parent_ref.lock().unwrap();
        let mut phy_display = parent.physical_display.lock().unwrap();

        let origin = self.aria.top_left;
        phy_display.draw_iter(pixels.into_iter().map(move |mut p| {
            p.0 += origin;
            p
        }))
    }

    fn fill_contiguous<I>(&mut self, area: &Rectangle, colors: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Self::Color>,
    {
        if !self.is_active {
            return Ok(());
        }

        let parent_ref = self.parent.clone();
        let parent = parent_ref.lock().unwrap();
        let mut phy_display = parent.physical_display.lock().unwrap();

        let origin = self.aria.top_left;
        // 过滤并填充
        phy_display.fill_contiguous(
            &Rectangle::new(origin + area.top_left, area.size),
            colors,
        )
    }

    fn fill_solid(&mut self, area: &Rectangle, color: Self::Color) -> Result<(), Self::Error> {
        if !self.is_active {
            return Ok(());
        }

        let parent_ref = self.parent.clone();
        let parent = parent_ref.lock().unwrap();
        let mut phy_display = parent.physical_display.lock().unwrap();

        let origin = self.aria.top_left;
        phy_display.fill_solid(&Rectangle::new(origin + area.top_left, area.size), color)
    }

    fn clear(&mut self, color: Self::Color) -> Result<(), Self::Error> {
        if !self.is_active {
            return Ok(());
        }

        let parent_ref = self.parent.clone();
        let parent = parent_ref.lock().unwrap();
        let mut phy_display = parent.physical_display.lock().unwrap();

        phy_display.clear(color)
    }
}

pub struct DisplayGroup<D>
where
    D: DrawTarget,
{
    physical_display: Arc<Mutex<D>>,
    physical_display_size: Size,
    logical_displays: Vec<Arc<Mutex<LogicalDisplay<D>>>>,
    current_active_display: isize,
}

impl<D> DisplayGroup<D>
where
    D: DrawTarget,
{
    pub fn new(physical_display: Arc<Mutex<D>>, initial_logical_display_capacity: usize) -> Self {
        info!("DisplayGroup::new");
        let physical_display_size = physical_display.lock().unwrap().bounding_box().size;
        Self {
            physical_display,
            physical_display_size,
            logical_displays: Vec::with_capacity(initial_logical_display_capacity),
            current_active_display: -1,
        }
    }

    pub fn get_physical_display_size(&self) -> Size {
        self.physical_display_size
    }

    pub fn get_current_active_display_index(&self) -> isize {
        self.current_active_display
    }

    pub fn switch_to_logical_display(&mut self, index: isize) -> Arc<Mutex<LogicalDisplay<D>>> {
        if index < 0 || index >= self.logical_displays.len() as isize {
            panic!("index out of range {}", index);
        }
        if self.current_active_display != -1 {
            let old_display_ref = &self.logical_displays[self.current_active_display as usize];
            let mut old_display = old_display_ref.lock().unwrap();
            old_display.is_active = false;
        }

        self.current_active_display = index;
        let new_display_ref = &self.logical_displays[self.current_active_display as usize];
        let mut new_display = new_display_ref.lock().unwrap();
        new_display.is_active = true;
        info!("switch to logical display {}", index);
        new_display_ref.clone()
    }
}
