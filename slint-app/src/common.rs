use std::{
    fmt::Debug,
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
    primitives::{PointsIter, Rectangle},
};

use log::{debug, info};

use embedded_graphics_group::{DisplayGroup, LogicalDisplay};

use crate::hsv::hsv_to_rgb;
use crate::FPSTestType;

#[derive(Debug, Clone, Copy)]
enum TestFPSAppEvent {
    UpdateType(FPSTestType),
    Exit,
}

pub struct FPSTestApp<EGC, EGD, EGE>
where
    EGC: PixelColor + 'static + From<Rgb888>,
    EGD: DrawTarget<Color = EGC, Error = EGE> + 'static,
    EGE: Debug,
{
    // 外部传递进来的字段
    display_group: Arc<Mutex<DisplayGroup<EGD>>>,

    // 内部使用字段
    display: Arc<Mutex<LogicalDisplay<EGD>>>,
    old_display_id: isize,
    new_display_id: usize,
    join_handle: Option<thread::JoinHandle<()>>,
    event_sender: mpsc::Sender<TestFPSAppEvent>,
    event_receiver: Arc<Mutex<mpsc::Receiver<TestFPSAppEvent>>>,
    aria: Rectangle,
}

impl<EGC, EGD, EGE> FPSTestApp<EGC, EGD, EGE>
where
    EGC: PixelColor + 'static + From<Rgb888>,
    EGD: DrawTarget<Color = EGC, Error = EGE> + 'static + Send,
    EGE: Debug,
{
    pub fn new(display_group: Arc<Mutex<DisplayGroup<EGD>>>) -> Self {
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
        let aria = display.lock().unwrap().get_aria();
        Self {
            old_display_id,
            display_group: display_group.clone(),
            display,
            new_display_id,
            join_handle: None,
            event_sender,
            event_receiver: Arc::new(Mutex::new(event_receiver)),
            aria,
        }
    }

    pub fn enter(&mut self) {
        info!("enter testfps app");
        // 切换到当前逻辑屏幕
        self.display_group
            .lock()
            .unwrap()
            .switch_to_logical_display(self.new_display_id as isize);

        let display_ref = self.display.clone();

        let recv_ref = self.event_receiver.clone();

        let aria = self.aria;
        self.join_handle = Some(thread::spawn(move || {
            let mut display = display_ref.lock().unwrap();
            let recv = recv_ref.lock().unwrap();

            let fps = Arc::new(Mutex::new(0));
            let fps_exit_signal = Arc::new(Mutex::new(false));
            let fps_exit_signal_ref = fps_exit_signal.clone();
            let fps_clone1 = fps.clone();
            let fps_clone2 = fps.clone();
            let fps_join_handler = thread::spawn(move || loop {
                let mut fps = fps_clone1.lock().unwrap();
                info!("fps: {}", *fps);
                *fps = 0;
                drop(fps);
                thread::sleep(Duration::from_secs(1));
                if *fps_exit_signal_ref.lock().unwrap() {
                    break;
                }
            });

            let mut current_type = FPSTestType::HSVFullScreen;
            let mut cnt = 0;
            loop {
                if let Ok(event) = recv.try_recv() {
                    info!("get event: {:?}", event);
                    match event {
                        TestFPSAppEvent::Exit => {
                            break;
                        }
                        TestFPSAppEvent::UpdateType(t) => {
                            current_type = t;
                        }
                    }
                }

                *fps_clone2.lock().unwrap() += 1;
                cnt = (cnt + 1) % 360;
                Self::draw_by_type(&mut display, aria, current_type, cnt as f32);

                
                thread::sleep(Duration::from_millis(10));
            }
            debug!("fps app thread will exit");
            *fps_exit_signal.lock().unwrap() = true;
            fps_join_handler.join().unwrap();
        }));
    }

    pub fn exit(&mut self) {
        info!("exit fpstest app");
        if self.join_handle.is_none() {
            return;
        }

        self.event_sender.send(TestFPSAppEvent::Exit).unwrap();

        thread::sleep(Duration::from_secs(5));
        debug!("wait for fpstest app thread exit");
        self.join_handle.take().unwrap().join().unwrap();
        debug!("fpstest app thread exited");

        self.display_group
            .lock()
            .unwrap()
            .switch_to_logical_display(self.old_display_id);
    }
}
