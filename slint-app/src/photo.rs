use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex, mpsc::{self, Sender},
    },
    thread,
    time::Duration,
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
use log::{debug, info};

use crate::ColorAdapter;
use embedded_graphics_group::{DisplayGroup, LogicalDisplay};

enum PhotoAppEvent {
    Next,
    AutoPlay,
    StopAutoPlay,
    Exit,
}

pub struct PhotoApp<C, EGC, EGD, ECA>
where
    C: Connection + 'static + Send,
    EGC: PixelColor + 'static,
    EGD: DrawTarget<Color = EGC> + 'static,
    ECA: ColorAdapter<Color = EGC> + 'static,
{
    // 外部传递进来的字段
    client: Arc<Mutex<Client<C>>>,
    display_group: Arc<Mutex<DisplayGroup<EGC, EGD>>>,
    color_adapter: ECA,

    // 内部使用字段
    display: Arc<Mutex<LogicalDisplay<EGC, EGD>>>,
    old_display_id: isize,
    new_display_id: usize,
    join_handle: Option<thread::JoinHandle<()>>,
    event_sender: mpsc::Sender<PhotoAppEvent>,
    event_receiver: Arc<Mutex<mpsc::Receiver<PhotoAppEvent>>>,
}

impl<C, EGC, EGD, ECA> PhotoApp<C, EGC, EGD, ECA>
where
    C: Connection + 'static + Send,
    EGC: PixelColor + 'static,
    EGD: DrawTarget<Color = EGC> + 'static + Send,
    ECA: ColorAdapter<Color = EGC> + 'static,
{
    pub fn new(
        client: Arc<Mutex<Client<C>>>,
        display_group: Arc<Mutex<DisplayGroup<EGC, EGD>>>,
        color_adapter: ECA,
    ) -> Self {
        let old_display_id = display_group
            .lock()
            .unwrap()
            .get_current_active_display_index();
        let physical_display_size = display_group.lock().unwrap().get_physical_display_size();
        let display = LogicalDisplay::new(
            display_group.clone(),
            Rectangle::new(Point::zero(), physical_display_size),
        );
        let new_display_id = display.lock().unwrap().get_id();
        let (event_sender, event_receiver) = mpsc::channel();
        Self {
            client,
            color_adapter,
            old_display_id,
            display_group: display_group.clone(),
            display,
            new_display_id,
            join_handle: None,
            event_sender,
            event_receiver: Arc::new(Mutex::new(event_receiver)),
        }
    }

    pub fn enter(&mut self) {
        info!("enter photo app");
        // 切换到当前逻辑屏幕
        self.display_group
            .lock()
            .unwrap()
            .switch_to_logical_display(self.new_display_id as isize);

        let display_ref = self.display.clone();
        let color_adapter = self.color_adapter;
        let client_ref = self.client.clone();
        let recv_ref = self.event_receiver.clone();
        self.join_handle = Some(thread::spawn(move || {
            let mut display = display_ref.lock().unwrap();
            let mut client = client_ref.lock().unwrap();
            let recv = recv_ref.lock().unwrap();
            let mut auto_play_mode = false;
            loop {
                if let Ok(event) = recv.try_recv() {
                    match event {
                        PhotoAppEvent::Exit => {
                            break;
                        },
                        PhotoAppEvent::Next => {
                            Self::load_image_to_screen(&mut client, &mut display, color_adapter);
                        }
                        PhotoAppEvent::AutoPlay => {
                            auto_play_mode = true;
                        }
                        PhotoAppEvent::StopAutoPlay => {
                            auto_play_mode = false;
                        }
                    }
                }
                if auto_play_mode {
                    Self::load_image_to_screen(&mut client, &mut display, color_adapter);
                }
                thread::sleep(Duration::from_millis(20));
            }
            debug!("photo app thread will exit");
        }));
    }

    pub fn next(&mut self) {
        info!("next photo");
        self.event_sender.send(PhotoAppEvent::Next).unwrap();
    }

    pub fn auto_play(&mut self) {
        info!("auto play photo");
        self.event_sender.send(PhotoAppEvent::AutoPlay).unwrap();
    }

    pub fn stop_auto_play(&mut self) {
        info!("stop auto play photo");
        self.event_sender.send(PhotoAppEvent::StopAutoPlay).unwrap();
    }

    pub fn exit(&mut self) {
        info!("exit photo app");
        if self.join_handle.is_none() {
            return;
        }
        
        self.event_sender.send(PhotoAppEvent::Exit).unwrap();
        debug!("wait for photo app thread exit");

        self.join_handle.take().unwrap().join().unwrap();
        debug!("photo app thread exited");

        self.display_group
            .lock()
            .unwrap()
            .switch_to_logical_display(self.old_display_id);
    }

    fn load_image_to_screen(
        client: &mut Client<C>,
        display: &mut LogicalDisplay<EGC, EGD>,
        color_adapter: ECA,
    ) {
        let req = client
            .request(Method::Get, "http://192.168.242.118:3000/api/photo", &[])
            .unwrap();
        let mut resp = req.submit().unwrap();
        let mut byte_buf = [0u8; 2];
        resp.read_exact(&mut byte_buf).unwrap();
        let width = byte_buf[0] as usize;
        let height = byte_buf[1] as usize;
        info!("read frame: {}x{}", width, height);

        let buf_lines = height / 1;
        let mut line_buf = vec![0u8; width * 3 * buf_lines];
        for i in 0..height / buf_lines {
            resp.read_exact(&mut line_buf).unwrap();
            let rect = &Rectangle {
                top_left: Point::new(0, i as i32 * buf_lines as i32),
                size: Size::new(width as u32, buf_lines as u32),
            };
            let colors = line_buf
                .chunks(3)
                .map(|rgb| color_adapter.rgb888(Rgb888::new(rgb[0], rgb[1], rgb[2])));
            _ = display.fill_contiguous(rect, colors);
        }
    }
}
