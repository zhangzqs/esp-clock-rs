use std::{cell::RefCell, fmt::Debug, rc::Rc, usize};

use embedded_graphics::{
    draw_target::DrawTarget,
    geometry::{OriginDimensions, Size},
    pixelcolor::{PixelColor, Rgb888},
    primitives::Rectangle,
    Pixel,
};
use log::info;

pub struct LogicalDisplay<D> {
    parent: Rc<RefCell<DisplayMux<D>>>,
    is_active: bool,
    id: usize,
    size: Size,
}

impl<D> LogicalDisplay<D>
where
    D: DrawTarget,
{
    pub fn new(parent: Rc<RefCell<DisplayMux<D>>>) -> Rc<RefCell<Self>> {
        let mut parent_mut_ref = parent.borrow_mut();
        let child_display = Self {
            parent: parent.clone(),
            is_active: false,
            id: parent_mut_ref.logical_displays.len(),
            size: parent_mut_ref.physical_display_size,
        };
        info!("create logical display {}", child_display.id);
        let child = Rc::new(RefCell::new(child_display));
        parent_mut_ref.logical_displays.push(child.clone());
        child
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
        self.size
    }
}

impl<C, D, E> DrawTarget for LogicalDisplay<D>
where
    C: PixelColor + From<Rgb888>,
    D: DrawTarget<Color = C, Error = E>,
    E: Debug,
{
    type Color = Rgb888;

    type Error = String;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        if !self.is_active {
            return Ok(());
        }

        let parent = self.parent.borrow();
        let mut phy_display = parent.physical_display.borrow_mut();
        phy_display
            .draw_iter(pixels.into_iter().map(|Pixel(p, c)| Pixel(p, c.into())))
            .map_err(|e| format!("{e:?}"))
    }

    fn fill_contiguous<I>(&mut self, area: &Rectangle, colors: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Self::Color>,
    {
        if !self.is_active {
            return Ok(());
        }

        let parent = self.parent.borrow();
        let mut phy_display = parent.physical_display.borrow_mut();

        // 过滤并填充
        phy_display
            .fill_contiguous(area, colors.into_iter().map(|x| x.into()))
            .map_err(|e| format!("{e:?}"))
    }

    fn fill_solid(&mut self, area: &Rectangle, color: Self::Color) -> Result<(), Self::Error> {
        if !self.is_active {
            return Ok(());
        }

        let parent = self.parent.borrow();
        let mut phy_display = parent.physical_display.borrow_mut();

        phy_display
            .fill_solid(area, color.into())
            .map_err(|e| format!("{e:?}"))
    }

    fn clear(&mut self, color: Self::Color) -> Result<(), Self::Error> {
        if !self.is_active {
            return Ok(());
        }

        let parent = self.parent.borrow();
        let mut phy_display = parent.physical_display.borrow_mut();

        phy_display
            .clear(color.into())
            .map_err(|e| format!("{e:?}"))
    }
}

pub struct DisplayMux<D> {
    physical_display: Rc<RefCell<D>>,
    physical_display_size: Size,
    logical_displays: Vec<Rc<RefCell<LogicalDisplay<D>>>>,
    current_active_display: isize,
}

impl<C, D, E> DisplayMux<D>
where
    C: From<Rgb888>,
    D: DrawTarget<Color = C, Error = E>,
    E: Debug,
{
    pub fn new(physical_display: Rc<RefCell<D>>, initial_logical_display_capacity: usize) -> Self {
        info!("DisplayMux::new");
        let physical_display_size = physical_display.borrow().bounding_box().size;
        Self {
            physical_display,
            physical_display_size,
            logical_displays: Vec::with_capacity(initial_logical_display_capacity),
            current_active_display: -1,
        }
    }

    pub fn size(&self) -> Size {
        self.physical_display_size
    }

    pub fn active_index(&self) -> isize {
        self.current_active_display
    }

    pub fn switch_to(&mut self, index: isize) -> Rc<RefCell<LogicalDisplay<D>>> {
        // index越界判定
        if index < 0 || index >= self.logical_displays.len() as isize {
            panic!("index out of range {}", index);
        }

        // 旧屏幕先取消激活状态
        if self.current_active_display != -1 {
            self.logical_displays[self.current_active_display as usize]
                .borrow_mut()
                .is_active = false;
        }

        // 更新 current_active_display
        self.current_active_display = index;
        info!("switch to logical display {}", index);

        // 新屏幕使能激活状态
        let new_display = &self.logical_displays[self.current_active_display as usize];
        new_display.borrow_mut().is_active = true;
        new_display.clone()
    }
}
