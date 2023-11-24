use std::{
    fmt::Debug,
    ops::AddAssign,
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
    pixelcolor::{PixelColor, Rgb888, RgbColor},
    primitives::{Circle, PointsIter, Primitive, PrimitiveStyle, Rectangle},
    Drawable,
};

use log::{debug, info};

use embedded_graphics_group::{DisplayGroup, LogicalDisplay};
use time::{Instant, OffsetDateTime, UtcOffset};

use crate::hsv::hsv_to_rgb;
use crate::FPSTestType;

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
    display_group: Arc<Mutex<DisplayGroup<EGC, EGD>>>,

    // 内部使用字段
    display: Arc<Mutex<LogicalDisplay<EGC, EGD>>>,
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
    pub fn new(display_group: Arc<Mutex<DisplayGroup<EGC, EGD>>>) -> Self {
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
            let fps_clone1 = fps.clone();
            let fps_clone2 = fps.clone();
            thread::spawn(move || loop {
                let mut fps = fps_clone1.lock().unwrap();
                info!("fps: {}", *fps);
                *fps = 0;
                drop(fps);
                thread::sleep(Duration::from_secs(1));
            });

            let mut current_type = FPSTestType::HSVFullScreen;
            let mut cnt = 0;
            loop {
                *fps_clone2.lock().unwrap() += 1;
                cnt = (cnt + 1) % 360;

                if let Ok(event) = recv.try_recv() {
                    match event {
                        TestFPSAppEvent::Exit => {
                            break;
                        }
                        TestFPSAppEvent::UpdateType(t) => {
                            current_type = t;
                        }
                    }
                }

                if current_type == FPSTestType::HSVFullScreen {
                    let (r, g, b) = hsv_to_rgb(cnt as f32, 1.0, 1.0);
                    display
                        .fill_solid(&aria, Rgb888::new(r, g, b).into())
                        .unwrap();
                } else {
                    let max_dist = (aria.size.width as f32).hypot(aria.size.height as f32) / 2.0;
                    display
                        .fill_contiguous(
                            &aria,
                            aria.points().map(|p| {
                                let (x, y) = (p - aria.center()).into();
                                let (x, y) = (x as f32, y as f32);
                                // 转换为极坐标
                                let (r, theta) = (x.hypot(y), y.atan2(x));
                                if theta.is_nan() {
                                    return Rgb888::new(0, 0, 0).into();
                                }
                                let mut deg = theta.to_degrees();
                                if deg < 0.0 {
                                    deg += 360.0;
                                }
                                let per = r / max_dist;
                                let (r, g, b) = match current_type {
                                    FPSTestType::HSVRadial1 => hsv_to_rgb(deg, 1.0, 1.0),
                                    FPSTestType::HSVRadial2 => hsv_to_rgb(deg, per, 1.0),
                                    FPSTestType::HSVRadial3 => hsv_to_rgb(deg, 1.0-per, 1.0),
                                    FPSTestType::HSVRadial4 => hsv_to_rgb(deg, 1.0, per),
                                    FPSTestType::HSVRadial5 => hsv_to_rgb(deg, 1.0, 1.0-per),
                                    _ => todo!(),
                                };
                                Rgb888::new(r, g, b).into()
                            }),
                        )
                        .unwrap();
                }

                thread::sleep(Duration::from_millis(10));
            }
            debug!("fps app thread will exit");
        }));
    }

    pub fn exit(&mut self) {
        info!("exit nes app");
        if self.join_handle.is_none() {
            return;
        }

        self.event_sender.send(TestFPSAppEvent::Exit).unwrap();
        debug!("wait for nes app thread exit");

        self.join_handle.take().unwrap().join().unwrap();
        debug!("nes app thread exited");

        self.display_group
            .lock()
            .unwrap()
            .switch_to_logical_display(self.old_display_id);
    }

    pub fn update_type(&mut self, t: FPSTestType) {
        info!("update type to {:?}", t);
        self.event_sender
            .send(TestFPSAppEvent::UpdateType(t))
            .unwrap();
    }
}
