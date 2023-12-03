use std::{
    fmt::Debug,
    io::{self, Read},
    sync::{
        mpsc::{self},
        Arc, Mutex,
    },
    thread,
    time::Duration,
};

use embedded_graphics::{
    draw_target::DrawTarget,
    geometry::Point,
    pixelcolor::{PixelColor, Rgb888},
    primitives::Rectangle,
};

use log::{debug, error, info};

use crate::AppWindow;
use embedded_graphics_group::{DisplayGroup, LogicalDisplay};
use slint::Weak;

enum ProjectorAppEvent {
    Exit,
}

pub struct ProjectorApp<EGC, EGD, EGE>
where
    EGC: PixelColor + 'static + From<Rgb888>,
    EGD: DrawTarget<Color = EGC, Error = EGE> + 'static,
    EGE: Debug,
{
    // 外部传递进来的字段
    display_group: Arc<Mutex<DisplayGroup<EGC, EGD>>>,

    // 内部使用字段
    old_display_id: isize,
    new_display_id: usize,
    join_handle: Option<thread::JoinHandle<()>>,
    event_sender: mpsc::Sender<ProjectorAppEvent>,
    event_receiver: Arc<Mutex<mpsc::Receiver<ProjectorAppEvent>>>,
    app: Weak<AppWindow>,
}

impl<EGC, EGD, EGE> ProjectorApp<EGC, EGD, EGE>
where
    EGC: PixelColor + 'static + From<Rgb888>,
    EGD: DrawTarget<Color = EGC, Error = EGE> + 'static + Send,
    EGE: Debug,
{
    pub fn new(display_group: Arc<Mutex<DisplayGroup<EGC, EGD>>>, app: Weak<AppWindow>) -> Self {
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
            old_display_id,
            display_group: display_group.clone(),
            new_display_id,
            join_handle: None,
            event_sender,
            event_receiver: Arc::new(Mutex::new(event_receiver)),
            app,
        }
    }

    pub fn enter(&mut self) {
        info!("enter projector app");
        let display_group_ref = self.display_group.clone();
        let new_display_id = self.new_display_id;
        let old_display_id = self.old_display_id;
        let recv_ref = self.event_receiver.clone();
        let app = self.app.clone();

        self.join_handle = Some(thread::spawn(move || {
            // 开启一个tcp服务，监听端口，接收数据
            let listener = std::net::TcpListener::bind("0.0.0.0:8081").unwrap();
            listener
                .set_nonblocking(true)
                .expect("Cannot set non-blocking");

            let ip = listener.local_addr().unwrap().ip().to_string();
            let port = listener.local_addr().unwrap().port();
            app.upgrade_in_event_loop(move |ui| {
                ui.set_projector_page_ip(ip.into());
                ui.set_projector_page_port(port as _);
            })
            .unwrap();

            for stream in listener.incoming() {
                if let Ok(event) = recv_ref.lock().unwrap().try_recv() {
                    match event {
                        ProjectorAppEvent::Exit => {
                            break;
                        }
                    }
                }
                // 等待客户端连接
                match stream {
                    Ok(s) => {
                        info!("new client connected");
                        Self::handle_client(
                            display_group_ref.clone(),
                            new_display_id,
                            old_display_id,
                            recv_ref.clone(),
                            s,
                        );
                    }
                    Err(ref e) if e.kind() == io::ErrorKind::WouldBlock => {
                        // 没有新的连接，继续等待
                        thread::sleep(Duration::from_millis(100));
                        continue;
                    }
                    Err(e) => panic!("encountered IO error: {e}"),
                }
            }
            info!("projector app thread will exit");
        }));
    }

    pub fn exit(&mut self) {
        info!("exit projector app");
        if self.join_handle.is_none() {
            return;
        }

        self.event_sender.send(ProjectorAppEvent::Exit).unwrap();
        debug!("wait for projector app thread exit");

        self.join_handle.take().unwrap().join().unwrap();
        debug!("projector app thread exited");

        self.display_group
            .lock()
            .unwrap()
            .switch_to_logical_display(self.old_display_id);
    }

    fn handle_client(
        display_group: Arc<Mutex<DisplayGroup<EGC, EGD>>>,
        new_display_id: usize,
        old_display_id: isize,
        recv: Arc<Mutex<mpsc::Receiver<ProjectorAppEvent>>>,
        mut stream: std::net::TcpStream,
    ) {
        info!("enter client loop");

        // 连接成功后切换到当前逻辑屏幕
        let display_ref = display_group
            .lock()
            .unwrap()
            .switch_to_logical_display(new_display_id as isize);
        let recv = recv.lock().unwrap();

        let mut y = 0;
        let rows = 20;
        let mut buf = vec![0u8; 240 * rows * 3];
        loop {
            if let Ok(event) = recv.try_recv() {
                match event {
                    ProjectorAppEvent::Exit => {
                        break;
                    }
                }
            }

            if let Err(e) = stream.read_exact(&mut buf) {
                error!("read from stream error: {}", e);
                break;
            }
            display_ref
                .lock()
                .unwrap()
                .fill_contiguous(
                    &Rectangle {
                        top_left: Point { x: 0, y: y as _ },
                        size: embedded_graphics::geometry::Size {
                            width: 240,
                            height: rows as _,
                        },
                    },
                    buf.chunks_exact(3).map(|p| {
                        let r = p[0];
                        let g = p[1];
                        let b = p[2];
                        Rgb888::new(r, g, b).into()
                    }),
                )
                .unwrap();
            y = (y + rows) % 240;
            thread::sleep(Duration::from_millis(10));
        }

        // 断开连接后切换到原来的逻辑屏幕
        display_group
            .lock()
            .unwrap()
            .switch_to_logical_display(old_display_id);
        info!("exit client loop");
    }
}
