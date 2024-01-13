use std::{
    fmt::Debug,
    marker::PhantomData,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Sender}, Arc, Mutex,
    },
    thread,
    time::{Duration, Instant},
};

use embedded_graphics::{
    draw_target::DrawTarget,
    geometry::Point,
    pixelcolor::{PixelColor, Rgb888},
    primitives::Rectangle,
};
use embedded_graphics_group::{DisplayGroup, LogicalDisplay};
use log::{debug, info};

pub trait IGraphicsApp {
    type Event: Debug;
    type State;

    fn name<'a>() -> &'a str {
        "Unknown Graphics App"
    }
    fn setup() -> Self::State;
    fn render<DisplayColor, DisplayError, Display>(state: &mut Self::State, display: &mut Display)
    where
        DisplayColor: PixelColor + From<Rgb888>,
        DisplayError: Debug,
        Display: DrawTarget<Color = DisplayColor, Error = DisplayError>;
    fn event(state: &mut Self::State, e: Self::Event);
}

pub struct GraphicsAppBase<EGD, Event, App> {
    // 外部传递进来的字段
    display_group: Arc<Mutex<DisplayGroup<EGD>>>,

    // 内部使用字段
    display: Arc<Mutex<LogicalDisplay<EGD>>>,
    old_display_id: isize,
    new_display_id: usize,
    join_handle: Option<thread::JoinHandle<()>>,

    event_sender: mpsc::Sender<Event>,
    event_receiver: Arc<Mutex<mpsc::Receiver<Event>>>,
    exit_signal: Arc<AtomicBool>,
    _p: PhantomData<App>,
}

impl<EGC, EGD, EGE, Event, App> GraphicsAppBase<EGD, Event, App>
where
    EGC: PixelColor + 'static + From<Rgb888>,
    EGD: DrawTarget<Color = EGC, Error = EGE> + 'static + Send,
    EGE: Debug,
    Event: Debug + Send + 'static,
    App: IGraphicsApp<Event = Event> + Send + 'static,
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
            exit_signal: Arc::new(AtomicBool::new(false)),
            _p: PhantomData,
        }
    }

    pub fn enter(&mut self) {
        info!("enter {} app", App::name());
        // 切换到当前逻辑屏幕
        self.display_group
            .lock()
            .unwrap()
            .switch_to_logical_display(self.new_display_id as isize);

        let display_ref = self.display.clone();

        let recv_ref = self.event_receiver.clone();
        let exit_signal = self.exit_signal.clone();
        self.join_handle = Some(thread::spawn(move || {
            exit_signal.store(false, Ordering::SeqCst);
            let recv = recv_ref.lock().unwrap();
            let mut frame_counter = 0;
            let mut last_instant = Instant::now();
            // setup
            let mut state = App::setup();
            while !exit_signal.load(Ordering::SeqCst) {
                let mut display = display_ref.lock().unwrap();

                // render
                App::render(&mut state, &mut *display);
                if let Ok(event) = recv.try_recv() {
                    info!("{} app get event: {:?}", App::name(), event);
                    // event
                    App::event(&mut state, event);
                }
                // frame counter
                frame_counter += 1;
                if frame_counter >= 60 {
                    // 计算一次平均fps
                    let dur = last_instant.elapsed();
                    info!(
                        "{} fps: {}",
                        App::name(),
                        frame_counter as f32 / dur.as_secs_f32()
                    );
                    last_instant = Instant::now();
                    frame_counter = 0;
                }
                thread::sleep(Duration::from_millis(16));
            }
            debug!("{} app thread will exit", App::name());
        }));
    }

    pub fn exit(&mut self) {
        info!("exit {} app", App::name());
        if self.join_handle.is_none() {
            return;
        }

        self.exit_signal.store(true, Ordering::SeqCst);
        debug!("wait for {} app thread exit", App::name());
        self.join_handle.take().unwrap().join().unwrap();
        debug!("{} app thread exited", App::name());

        self.display_group
            .lock()
            .unwrap()
            .switch_to_logical_display(self.old_display_id);
    }

    pub fn send_event(&mut self, e: Event) {
        self.event_sender.send(e).unwrap();
    }

    pub fn get_sender(&mut self) -> Sender<Event> {
        self.event_sender.clone()
    }
}
