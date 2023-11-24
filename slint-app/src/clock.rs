const DIGIST_SHAPE_DATA: [[u8; 10]; 10] = [
    [28, 54, 99, 99, 99, 99, 99, 99, 54, 28],
    [12, 60, 12, 12, 12, 12, 12, 12, 12, 127],
    [62, 99, 3, 6, 12, 24, 48, 96, 99, 127],
    [127, 3, 6, 12, 28, 6, 3, 3, 99, 62],
    [6, 14, 30, 54, 102, 127, 6, 6, 6, 15],
    [127, 96, 96, 126, 3, 3, 3, 3, 99, 62],
    [6, 24, 48, 96, 110, 99, 99, 99, 99, 62],
    [127, 99, 6, 6, 12, 12, 24, 24, 24, 24],
    [62, 99, 99, 99, 62, 99, 99, 99, 99, 62],
    [62, 99, 99, 99, 59, 3, 3, 6, 12, 48],
];

pub fn get_digist_shape(digist: u8) -> [[bool; 7]; 10] {
    let mut ret: [[bool; 7]; 10] = [[false; 7]; 10];
    let shape = DIGIST_SHAPE_DATA[digist as usize];
    for i in 0..10 {
        for j in 0..7 {
            ret[i][j] = shape[i] & (1 << (6 - j)) != 0;
        }
    }
    ret
}

pub fn get_colon_shape() -> [[bool; 4]; 10] {
    let mut ret: [[bool; 4]; 10] = [[false; 4]; 10];
    for i in 0..10 {
        for j in 0..4 {
            ret[i][j] = (i == 2 || i == 3 || i == 6 || i == 7) && (j == 1 || j == 2);
        }
    }
    ret
}

pub fn hsv_to_rgb(hue: f64, saturation: f64, value: f64) -> (u8, u8, u8) {
    fn is_between(value: f64, min: f64, max: f64) -> bool {
        min <= value && value < max
    }

    check_bounds(hue, saturation, value);

    let c = value * saturation;
    let h = hue / 60.0;
    let x = c * (1.0 - ((h % 2.0) - 1.0).abs());
    let m = value - c;

    let (r, g, b): (f64, f64, f64) = if is_between(h, 0.0, 1.0) {
        (c, x, 0.0)
    } else if is_between(h, 1.0, 2.0) {
        (x, c, 0.0)
    } else if is_between(h, 2.0, 3.0) {
        (0.0, c, x)
    } else if is_between(h, 3.0, 4.0) {
        (0.0, x, c)
    } else if is_between(h, 4.0, 5.0) {
        (x, 0.0, c)
    } else {
        (c, 0.0, x)
    };

    (
        ((r + m) * 255.0) as u8,
        ((g + m) * 255.0) as u8,
        ((b + m) * 255.0) as u8,
    )
}

fn check_bounds(hue: f64, saturation: f64, value: f64) {
    fn panic_bad_params(name: &str, from_value: &str, to_value: &str, supplied: f64) -> ! {
        panic!(
            "param {} must be between {} and {} inclusive; was: {}",
            name, from_value, to_value, supplied
        )
    }

    if !(0.0..=360.0).contains(&hue) {
        panic_bad_params("hue", "0.0", "360.0", hue)
    } else if !(0.0..=1.0).contains(&saturation) {
        panic_bad_params("saturation", "0.0", "1.0", saturation)
    } else if !(0.0..=1.0).contains(&value) {
        panic_bad_params("value", "0.0", "1.0", value)
    }
}

mod tests {
    use super::*;

    const EXPECTED_COLON_SHAPE_DATA: [[u8; 4]; 10] = [
        [0, 0, 0, 0],
        [0, 0, 0, 0],
        [0, 1, 1, 0],
        [0, 1, 1, 0],
        [0, 0, 0, 0],
        [0, 0, 0, 0],
        [0, 1, 1, 0],
        [0, 1, 1, 0],
        [0, 0, 0, 0],
        [0, 0, 0, 0],
    ]; //:
    const EXPECTED_DIGIST_SHAPE_DATA: [[[u8; 7]; 10]; 10] = [
        [
            [0, 0, 1, 1, 1, 0, 0],
            [0, 1, 1, 0, 1, 1, 0],
            [1, 1, 0, 0, 0, 1, 1],
            [1, 1, 0, 0, 0, 1, 1],
            [1, 1, 0, 0, 0, 1, 1],
            [1, 1, 0, 0, 0, 1, 1],
            [1, 1, 0, 0, 0, 1, 1],
            [1, 1, 0, 0, 0, 1, 1],
            [0, 1, 1, 0, 1, 1, 0],
            [0, 0, 1, 1, 1, 0, 0],
        ], //0
        [
            [0, 0, 0, 1, 1, 0, 0],
            [0, 1, 1, 1, 1, 0, 0],
            [0, 0, 0, 1, 1, 0, 0],
            [0, 0, 0, 1, 1, 0, 0],
            [0, 0, 0, 1, 1, 0, 0],
            [0, 0, 0, 1, 1, 0, 0],
            [0, 0, 0, 1, 1, 0, 0],
            [0, 0, 0, 1, 1, 0, 0],
            [0, 0, 0, 1, 1, 0, 0],
            [1, 1, 1, 1, 1, 1, 1],
        ], //1
        [
            [0, 1, 1, 1, 1, 1, 0],
            [1, 1, 0, 0, 0, 1, 1],
            [0, 0, 0, 0, 0, 1, 1],
            [0, 0, 0, 0, 1, 1, 0],
            [0, 0, 0, 1, 1, 0, 0],
            [0, 0, 1, 1, 0, 0, 0],
            [0, 1, 1, 0, 0, 0, 0],
            [1, 1, 0, 0, 0, 0, 0],
            [1, 1, 0, 0, 0, 1, 1],
            [1, 1, 1, 1, 1, 1, 1],
        ], //2
        [
            [1, 1, 1, 1, 1, 1, 1],
            [0, 0, 0, 0, 0, 1, 1],
            [0, 0, 0, 0, 1, 1, 0],
            [0, 0, 0, 1, 1, 0, 0],
            [0, 0, 1, 1, 1, 0, 0],
            [0, 0, 0, 0, 1, 1, 0],
            [0, 0, 0, 0, 0, 1, 1],
            [0, 0, 0, 0, 0, 1, 1],
            [1, 1, 0, 0, 0, 1, 1],
            [0, 1, 1, 1, 1, 1, 0],
        ], //3
        [
            [0, 0, 0, 0, 1, 1, 0],
            [0, 0, 0, 1, 1, 1, 0],
            [0, 0, 1, 1, 1, 1, 0],
            [0, 1, 1, 0, 1, 1, 0],
            [1, 1, 0, 0, 1, 1, 0],
            [1, 1, 1, 1, 1, 1, 1],
            [0, 0, 0, 0, 1, 1, 0],
            [0, 0, 0, 0, 1, 1, 0],
            [0, 0, 0, 0, 1, 1, 0],
            [0, 0, 0, 1, 1, 1, 1],
        ], //4
        [
            [1, 1, 1, 1, 1, 1, 1],
            [1, 1, 0, 0, 0, 0, 0],
            [1, 1, 0, 0, 0, 0, 0],
            [1, 1, 1, 1, 1, 1, 0],
            [0, 0, 0, 0, 0, 1, 1],
            [0, 0, 0, 0, 0, 1, 1],
            [0, 0, 0, 0, 0, 1, 1],
            [0, 0, 0, 0, 0, 1, 1],
            [1, 1, 0, 0, 0, 1, 1],
            [0, 1, 1, 1, 1, 1, 0],
        ], //5
        [
            [0, 0, 0, 0, 1, 1, 0],
            [0, 0, 1, 1, 0, 0, 0],
            [0, 1, 1, 0, 0, 0, 0],
            [1, 1, 0, 0, 0, 0, 0],
            [1, 1, 0, 1, 1, 1, 0],
            [1, 1, 0, 0, 0, 1, 1],
            [1, 1, 0, 0, 0, 1, 1],
            [1, 1, 0, 0, 0, 1, 1],
            [1, 1, 0, 0, 0, 1, 1],
            [0, 1, 1, 1, 1, 1, 0],
        ], //6
        [
            [1, 1, 1, 1, 1, 1, 1],
            [1, 1, 0, 0, 0, 1, 1],
            [0, 0, 0, 0, 1, 1, 0],
            [0, 0, 0, 0, 1, 1, 0],
            [0, 0, 0, 1, 1, 0, 0],
            [0, 0, 0, 1, 1, 0, 0],
            [0, 0, 1, 1, 0, 0, 0],
            [0, 0, 1, 1, 0, 0, 0],
            [0, 0, 1, 1, 0, 0, 0],
            [0, 0, 1, 1, 0, 0, 0],
        ], //7
        [
            [0, 1, 1, 1, 1, 1, 0],
            [1, 1, 0, 0, 0, 1, 1],
            [1, 1, 0, 0, 0, 1, 1],
            [1, 1, 0, 0, 0, 1, 1],
            [0, 1, 1, 1, 1, 1, 0],
            [1, 1, 0, 0, 0, 1, 1],
            [1, 1, 0, 0, 0, 1, 1],
            [1, 1, 0, 0, 0, 1, 1],
            [1, 1, 0, 0, 0, 1, 1],
            [0, 1, 1, 1, 1, 1, 0],
        ], //8
        [
            [0, 1, 1, 1, 1, 1, 0],
            [1, 1, 0, 0, 0, 1, 1],
            [1, 1, 0, 0, 0, 1, 1],
            [1, 1, 0, 0, 0, 1, 1],
            [0, 1, 1, 1, 0, 1, 1],
            [0, 0, 0, 0, 0, 1, 1],
            [0, 0, 0, 0, 0, 1, 1],
            [0, 0, 0, 0, 1, 1, 0],
            [0, 0, 0, 1, 1, 0, 0],
            [0, 1, 1, 0, 0, 0, 0],
        ], //9
    ];

    #[test]
    fn test() {
        // 将数字转换为二进制数组
        for i in 0..10 {
            let actual = get_digist_shape(i)
                .iter()
                .map(|x| x.to_vec())
                .collect::<Vec<Vec<bool>>>();
            let expected = EXPECTED_DIGIST_SHAPE_DATA[i as usize]
                .iter()
                .map(|x| x.iter().map(|x| *x == 1).collect::<Vec<bool>>())
                .collect::<Vec<Vec<bool>>>();
            assert_eq!(actual, expected);
        }
    }
}

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
    geometry::{Point, Size},
    pixelcolor::{PixelColor, Rgb888, RgbColor},
    primitives::{Circle, Primitive, PrimitiveStyle, Rectangle, PointsIter},
    Drawable,
};

use log::{debug, info};

use embedded_graphics_group::{DisplayGroup, LogicalDisplay};
use time::{Instant, OffsetDateTime, UtcOffset};

enum ClockAppEvent {
    Exit,
}

pub struct ClockApp<EGC, EGD, EGE>
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
    event_sender: mpsc::Sender<ClockAppEvent>,
    event_receiver: Arc<Mutex<mpsc::Receiver<ClockAppEvent>>>,
}

struct ClockAppState {
    digist_cache: [u8; 6],
    has_init: bool,
}

impl<EGC, EGD, EGE> ClockApp<EGC, EGD, EGE>
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
        Self {
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
        info!("enter nes app");
        // 切换到当前逻辑屏幕
        self.display_group
            .lock()
            .unwrap()
            .switch_to_logical_display(self.new_display_id as isize);

        let display_ref = self.display.clone();

        let recv_ref = self.event_receiver.clone();
        

        self.join_handle = Some(thread::spawn(move || {
            let mut display = display_ref.lock().unwrap();
            let recv = recv_ref.lock().unwrap();
            let mut state = ClockAppState {
                digist_cache: [0; 6],
                has_init: false,
            };

            let fps = Arc::new(Mutex::new(0));
            let fps_clone1 = fps.clone();
            let fps_clone2 = fps.clone();    
            thread::spawn(move || {
                loop {
                    let mut fps = fps_clone1.lock().unwrap();
                    info!("fps: {}", *fps);
                    *fps = 0;
                    drop(fps);
                    thread::sleep(Duration::from_secs(1));
                }
            });
            let mut hue = 0;
            loop {
                let mut fps = fps_clone2.lock().unwrap();
                *fps += 1;
                drop(fps);
                hue = (hue + 1)%360;
                let (r, g, b) = hsv_to_rgb(hue as f64, 1.0, 1.0);
                let aria = Rectangle {
                    top_left: Point::new(0, 0),
                    size: Size::new(240, 240),
                };
                // 显示白色表示初始化完成
                display
                    .fill_solid(&aria, Rgb888::new(r, g, b).into())
                    .unwrap();
                // Self::app_loop(&mut *display, &mut state);
                if let Ok(event) = recv.try_recv() {
                    match event {
                        ClockAppEvent::Exit => {
                            break;
                        }
                    }
                }
                thread::sleep(Duration::from_millis(10));
            }
            debug!("nes app thread will exit");
        }));
    }

    pub fn exit(&mut self) {
        info!("exit nes app");
        if self.join_handle.is_none() {
            return;
        }

        self.event_sender.send(ClockAppEvent::Exit).unwrap();
        debug!("wait for nes app thread exit");

        self.join_handle.take().unwrap().join().unwrap();
        debug!("nes app thread exited");

        self.display_group
            .lock()
            .unwrap()
            .switch_to_logical_display(self.old_display_id);
    }

    fn app_loop(display: &mut LogicalDisplay<EGC, EGD>, state: &mut ClockAppState) {
        let t = OffsetDateTime::now_utc().to_offset(UtcOffset::from_hms(8, 0, 0).unwrap());
        let (h, m, s) = (t.hour(), t.minute(), t.second());
        let (h0, h1) = (h / 10, h % 10);
        let (m0, m1) = (m / 10, m % 10);
        let (s0, s1) = (s / 10, s % 10);

        // dx为左上角x坐标，dy为左上角y坐标，r为圆半径，gap为水平或垂直两圆之间的圆心距离
        let draw_digist = |display: &mut LogicalDisplay<EGC, EGD>,
                           digist: u8,
                           dx: i32,
                           dy: i32,
                           r: i32,
                           gap: i32| {
            let shape = get_digist_shape(digist);
            for y in 0..10 {
                for x in 0..7 {
                    if shape[y][x] {
                        Circle::new(
                            Point::new(dx + x as i32 * gap, dy + y as i32 * gap),
                            2 * r as u32,
                        )
                        .into_styled(PrimitiveStyle::with_fill({
                            let (r, g, b) = hsv_to_rgb(
                                360.0 * (dx + x as i32) as f64
                                    / (display.get_aria().size.width as i32) as f64,
                                1.0,
                                1.0,
                            );
                            Rgb888::new(r, g, b).into()
                        }))
                        .draw(display)
                        .unwrap();
                    }
                }
            }
        };
        let clear_digit = |display: &mut LogicalDisplay<EGC, EGD>, dx: i32, dy: i32, gap: i32| {
            display.fill_solid(
                &Rectangle::new(
                    Point::new(dx, dy),
                    Size::new(7 * gap as u32, 10 * gap as u32),
                ),
                Rgb888::BLACK.into(),
            ).unwrap();
        };

        let single_digist_width = 38;

        if !state.has_init {
            state.has_init = true;
            state.digist_cache = [h0, h1, m0, m1, s0, s1];
            display.clear(Rgb888::BLACK.into()).unwrap();

            for i in 0..6 {
                draw_digist(display, state.digist_cache[i], single_digist_width * i as i32, 100, 2, 5);
            }
        } else {
            // 局部按需
            for i in 0..6 {
                if state.digist_cache[i] != [h0, h1, m0, m1, s0, s1][i] {
                    state.digist_cache[i] = [h0, h1, m0, m1, s0, s1][i];
                    clear_digit(display, single_digist_width * i as i32, 100, 5);
                    draw_digist(display, state.digist_cache[i], single_digist_width * i as i32, 100, 2, 5);
                }
            }
        }
    }
}
