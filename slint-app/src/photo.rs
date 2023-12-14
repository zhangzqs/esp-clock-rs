use std::{
    error,
    sync::{
        mpsc::{self},
        Arc, Mutex,
    },
    thread,
    time::Duration,
};

use embedded_graphics::{
    draw_target::DrawTarget,
    geometry::{Point, Size},
    pixelcolor::{PixelColor, Rgb888},
    primitives::Rectangle,
};
use embedded_svc::{
    http::{
        client::{Client, Connection},
        Method,
    },
    io::Read,
};
use log::{debug, error, info};

use embedded_graphics_group::{DisplayGroup, LogicalDisplay};

enum PhotoAppEvent {
    Next,
    AutoPlay,
    StopAutoPlay,
    Exit,
}

pub struct PhotoApp<CONN, ConnErr, EGC, EGD>
where
    CONN: Connection<Error = ConnErr> + 'static + Send,
    ConnErr: error::Error + 'static,
    EGC: PixelColor + 'static + From<Rgb888>,
    EGD: DrawTarget<Color = EGC> + 'static,
{
    // 外部传递进来的字段
    client: Arc<Mutex<Client<CONN>>>,
    display_group: Arc<Mutex<DisplayGroup<EGD>>>,

    // 内部使用字段
    display: Arc<Mutex<LogicalDisplay<EGD>>>,
    old_display_id: isize,
    new_display_id: usize,
    join_handle: Option<thread::JoinHandle<()>>,
    event_sender: mpsc::SyncSender<PhotoAppEvent>,
    event_receiver: Arc<Mutex<mpsc::Receiver<PhotoAppEvent>>>,
}

impl<CONN, ConnErr, EGC, EGD> PhotoApp<CONN, ConnErr, EGC, EGD>
where
    CONN: Connection<Error = ConnErr> + 'static + Send,
    ConnErr: error::Error + 'static,
    EGC: PixelColor + 'static + From<Rgb888>,
    EGD: DrawTarget<Color = EGC> + 'static + Send,
{
    pub fn new(
        client: Arc<Mutex<Client<CONN>>>,
        display_group: Arc<Mutex<DisplayGroup<EGD>>>,
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
        let (event_sender, event_receiver) = mpsc::sync_channel(2);
        Self {
            client,
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
                        }
                        PhotoAppEvent::Next => {
                            Self::load_image_to_screen(&mut client, &mut display);
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
                    if Self::load_image_to_screen(&mut client, &mut display) {
                        // 加载成功，等待 2s
                        thread::sleep(Duration::from_secs(2));
                    } else {
                        error!("load image failed");
                    }
                }
                thread::sleep(Duration::from_millis(50));
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

    fn load_image_to_screen(client: &mut Client<CONN>, display: &mut LogicalDisplay<EGD>) -> bool {
        let req = client.request(Method::Get, "http://192.168.242.118:3000/api/photo", &[]);
        if let Err(e) = req {
            error!("create request failed: {}", e);
            return false;
        }
        let req = req.unwrap();
        let resp = req.submit();
        if let Err(e) = resp {
            error!("submit request failed: {}", e);
            return false;
        }
        let mut resp = resp.unwrap();
        let mut byte_buf = [0u8; 2];
        if let Err(e) = resp.read_exact(&mut byte_buf) {
            error!("read frame size failed: {}", e);
            return false;
        }
        let width = byte_buf[0] as usize;
        let height = byte_buf[1] as usize;
        info!("read frame: {}x{}", width, height);

        let buf_lines = height / height;
        let mut line_buf = vec![0u8; width * 3 * buf_lines];
        for i in 0..height / buf_lines {
            if let Err(e) = resp.read_exact(&mut line_buf) {
                error!("read frame failed: {}", e);
                return false;
            }
            let rect = &Rectangle {
                top_left: Point::new(0, i as i32 * buf_lines as i32),
                size: Size::new(width as u32, buf_lines as u32),
            };
            let colors = line_buf
                .chunks(3)
                .map(|rgb| Rgb888::new(rgb[0], rgb[1], rgb[2]).into());
            _ = display.fill_contiguous(rect, colors);
        }
        true
    }
}
