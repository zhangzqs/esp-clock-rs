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

#[cfg(test)]
mod tests {
    use super::{get_digist_shape, get_colon_shape};
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
        // 冒号判定
        let actual = get_colon_shape()
            .iter()
            .map(|x| x.to_vec())
            .collect::<Vec<Vec<bool>>>();
        let expected = EXPECTED_COLON_SHAPE_DATA
            .iter()
            .map(|x| x.iter().map(|x| *x == 1).collect::<Vec<bool>>())
            .collect::<Vec<Vec<bool>>>();
        assert_eq!(actual, expected);
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
    primitives::{Circle, Primitive, PrimitiveStyle, Rectangle},
    Drawable,
};

use log::{debug, info};

use embedded_graphics_group::{DisplayGroup, LogicalDisplay};
use time::{OffsetDateTime, UtcOffset};

use crate::util::hsv_to_rgb;

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
    display_group: Arc<Mutex<DisplayGroup<EGD>>>,

    // 内部使用字段
    display: Arc<Mutex<LogicalDisplay<EGD>>>,
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
        info!("enter clock app");
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

            loop {
                Self::app_loop(&mut *display, &mut state);
                if let Ok(event) = recv.try_recv() {
                    match event {
                        ClockAppEvent::Exit => {
                            break;
                        }
                    }
                }
                thread::sleep(Duration::from_millis(10));
            }
            debug!("clock app thread will exit");
        }));
    }

    pub fn exit(&mut self) {
        info!("exit clock app");
        if self.join_handle.is_none() {
            return;
        }

        self.event_sender.send(ClockAppEvent::Exit).unwrap();
        debug!("wait for clock app thread exit");

        self.join_handle.take().unwrap().join().unwrap();
        debug!("clock app thread exited");

        self.display_group
            .lock()
            .unwrap()
            .switch_to_logical_display(self.old_display_id);
    }

    fn app_loop(display: &mut LogicalDisplay<EGD>, state: &mut ClockAppState) {
        let t = OffsetDateTime::now_utc().to_offset(UtcOffset::from_hms(8, 0, 0).unwrap());
        let (h, m, s) = (t.hour(), t.minute(), t.second());
        let (h0, h1) = (h / 10, h % 10);
        let (m0, m1) = (m / 10, m % 10);
        let (s0, s1) = (s / 10, s % 10);

        // dx为左上角x坐标，dy为左上角y坐标，r为圆半径，gap为水平或垂直两圆之间的圆心距离
        let draw_digist = |display: &mut LogicalDisplay<EGD>,
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
                                360.0 * (dx + x as i32) as f32
                                    / (display.get_aria().size.width as i32) as f32,
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
        let clear_digit = |display: &mut LogicalDisplay<EGD>, dx: i32, dy: i32, gap: i32| {
            display
                .fill_solid(
                    &Rectangle::new(
                        Point::new(dx, dy),
                        Size::new(7 * gap as u32, 10 * gap as u32),
                    ),
                    Rgb888::BLACK.into(),
                )
                .unwrap();
        };

        let single_digist_width = 38;

        if !state.has_init {
            state.has_init = true;
            state.digist_cache = [h0, h1, m0, m1, s0, s1];
            display.clear(Rgb888::BLACK.into()).unwrap();

            for i in 0..6 {
                draw_digist(
                    display,
                    state.digist_cache[i],
                    single_digist_width * i as i32,
                    100,
                    2,
                    5,
                );
            }
        } else {
            // 局部按需
            for i in 0..6 {
                if state.digist_cache[i] != [h0, h1, m0, m1, s0, s1][i] {
                    state.digist_cache[i] = [h0, h1, m0, m1, s0, s1][i];
                    clear_digit(display, single_digist_width * i as i32, 100, 5);
                    draw_digist(
                        display,
                        state.digist_cache[i],
                        single_digist_width * i as i32,
                        100,
                        2,
                        5,
                    );
                }
            }
        }
    }
}
