use std::{
    sync::{Arc, Mutex},
    thread, time::Duration,
};

use embedded_graphics::{
    draw_target::DrawTarget,
    geometry::{Point, Size},
    pixelcolor::{PixelColor, Rgb888, RgbColor},
    primitives::Rectangle,
};
use embedded_svc::{
    http::{
        client::{Client, Connection},
        Method,
    },
    io::Read,
};
use log::info;

use crate::ColorAdapter;
use embedded_graphics_group::{DisplayGroup, LogicalDisplay};

pub struct PhotoApp<C, EGC, EGD, ECA>
where
    C: Connection + 'static,
    EGC: PixelColor + 'static,
    EGD: DrawTarget<Color = EGC> + 'static,
    ECA: ColorAdapter<Color = EGC> + 'static,
{
    client: Arc<Mutex<Client<C>>>,
    display_group: Arc<Mutex<DisplayGroup<EGC, EGD>>>,
    color_adapter: ECA,

    display: Option<Arc<Mutex<LogicalDisplay<EGC, EGD>>>>,
}

impl<C, EGC, EGD, ECA> PhotoApp<C, EGC, EGD, ECA>
where
    C: Connection + 'static,
    EGC: PixelColor + 'static,
    EGD: DrawTarget<Color = EGC> + 'static + Send,
    ECA: ColorAdapter<Color = EGC> + 'static,
{
    pub fn new(
        client: Arc<Mutex<Client<C>>>,
        display_group: Arc<Mutex<DisplayGroup<EGC, EGD>>>,
        color_adapter: ECA,
    ) -> Self {
        Self {
            client,
            display_group,
            color_adapter,
            display: None,
        }
    }
    pub fn enter(&mut self) {
        info!("enter photo app");
        let mut display_group = self.display_group.lock().unwrap();
        let display = display_group.switch_to_logical_display(1);
        self.display = Some(display.clone());
        let color_adapter = self.color_adapter;
        thread::spawn(move || {
            let mut display = display.lock().unwrap();
            for i in (0..255).chain(255..0).cycle() {
                let color = color_adapter.rgb888(Rgb888::new(i, 0, 0));
                _ = display.clear(color);
                thread::sleep(Duration::from_millis(20));
            }
        });
    }

    pub fn exit(&self) {
        info!("exit photo app")
        // todo: stop thread
    }

    pub fn load_image_to_screen(&mut self) {
        // let red = self.color_adapter.rgb888(Rgb888::RED);
        // _ = self.display.clear(red);
        let mut client = self.client.lock().unwrap();
        let req = client
            .request(Method::Get, "http://192.168.242.118:3000/api/photo", &[])
            .unwrap();
        let mut resp = req.submit().unwrap();
        let mut byte_buf = [0u8; 2];
        resp.read_exact(&mut byte_buf).unwrap();
        let width = byte_buf[0] as usize;
        let height = byte_buf[1] as usize;
        info!("read frame: {}x{}", width, height);

        let buf_lines = height / 10;
        let mut line_buf = vec![0u8; width * 3 * buf_lines];
        let display = self.display.clone().unwrap();
        for i in 0..height / buf_lines {
            resp.read_exact(&mut line_buf).unwrap();
            let rect = &Rectangle {
                top_left: Point::new(0, i as i32 * buf_lines as i32),
                size: Size::new(width as u32, buf_lines as u32),
            };
            let colors = line_buf.chunks(3).map(|rgb| {
                self.color_adapter
                    .rgb888(Rgb888::new(rgb[0], rgb[1], rgb[2]))
            });
            _ = display.lock().unwrap().fill_contiguous(rect, colors);
        }
    }
}
