use std::{
    cell::RefCell,
    collections::HashMap,
    io::Read,
    rc::Rc,
    sync::{mpsc, Arc, Mutex},
    thread,
    time::Duration,
};

use app_core::{get_scheduler, proto::*};
use button_driver::Button;
use display_interface_spi::SPIInterface;
use embedded_hal::spi::MODE_3;
use embedded_io_adapters::std::ToStd;
use embedded_svc::http::client::Client;
use esp_idf_hal::{
    delay::FreeRtos,
    gpio::{AnyIOPin, Input, Pin, PinDriver},
    ledc::{config::TimerConfig, LedcDriver, LedcTimerDriver},
    peripheral,
    prelude::*,
    spi::{config::Config, Dma, SpiDeviceDriver, SpiDriverConfig},
};
use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    http::client::{Configuration, EspHttpConnection},
    nvs::EspDefaultNvsPartition,
    sntp::{EspSntp, OperatingMode, SntpConf, SyncMode, SyncStatus},
    wifi::{
        AsyncWifi, AuthMethod, BlockingWifi, ClientConfiguration,
        Configuration as WifiConfiguration, EspWifi,
    },
};
use esp_idf_sys as _;
use log::info;
use mipidsi::{Builder, ColorInversion, Orientation};

struct EspOneButton<'a, P: Pin> {
    button: Rc<RefCell<Button<PinDriver<'a, P, Input>, button_driver::DefaultPlatform>>>,
    timer: slint::Timer,
}

impl<'a, P: Pin> EspOneButton<'a, P> {
    fn new(pin: PinDriver<'a, P, Input>) -> Self {
        let button = button_driver::Button::new(pin, Default::default());
        Self {
            button: Rc::new(RefCell::new(button)),
            timer: slint::Timer::default(),
        }
    }
}

impl<'a: 'static, P: Pin> Node for EspOneButton<'a, P> {
    fn node_name(&self) -> NodeName {
        NodeName::Other("EspOneButton".into())
    }

    fn handle_message(&self, ctx: Rc<dyn Context>, msg: MessageWithHeader) -> HandleResult {
        if let Message::Lifecycle(LifecycleMessage::Init) = msg.body {
            let button = self.button.clone();
            self.timer.start(
                slint::TimerMode::Repeated,
                Duration::from_millis(20),
                move || {
                    let mut button = button.borrow_mut();
                    button.tick();

                    if button.clicks() > 0 {
                        let clicks = button.clicks();
                        if clicks == 1 {
                            ctx.boardcast(Message::OneButton(OneButtonMessage::Click));
                        } else {
                            ctx.boardcast(Message::OneButton(OneButtonMessage::Clicks(clicks)));
                        }
                    } else if let Some(dur) = button.current_holding_time() {
                        info!("Held for {dur:?}");
                        ctx.boardcast(Message::OneButton(OneButtonMessage::LongPressHolding(dur)));
                    } else if let Some(dur) = button.held_time() {
                        info!("Total holding time {dur:?}");
                        ctx.boardcast(Message::OneButton(OneButtonMessage::LongPressHeld(dur)));
                    }
                    button.reset();
                },
            );
        }
        HandleResult::Discard
    }
}

struct PerformanceNode {}

impl Node for PerformanceNode {
    fn node_name(&self) -> NodeName {
        NodeName::Performance
    }

    fn handle_message(&self, _ctx: Rc<dyn Context>, msg: MessageWithHeader) -> HandleResult {
        if let Message::Performance(pm) = msg.body {
            return HandleResult::Finish(Message::Performance(match pm {
                PerformanceMessage::GetFreeHeapSizeRequest => {
                    PerformanceMessage::GetFreeHeapSizeResponse(unsafe {
                        esp_idf_sys::esp_get_free_heap_size() as usize
                    })
                }
                PerformanceMessage::GetLargestFreeBlock => {
                    PerformanceMessage::GetLargestFreeBlockResponse(unsafe {
                        esp_idf_sys::heap_caps_get_largest_free_block(esp_idf_sys::MALLOC_CAP_8BIT)
                    })
                }
                PerformanceMessage::GetFpsRequest => PerformanceMessage::GetFpsResponse(60),
                m => panic!("unexpected message {m:?}"),
            }));
        }
        HandleResult::Discard
    }
}

struct WiFiNode<M> {
    sysloop: EspSystemEventLoop,
    modem: RefCell<Option<M>>,
    ready_resp: Arc<Mutex<HashMap<usize, Message>>>,
    nvs: EspDefaultNvsPartition,
}

impl<M> WiFiNode<M> {
    fn new(nvs: EspDefaultNvsPartition, sysloop: EspSystemEventLoop, modem: M) -> Self {
        Self {
            nvs,
            sysloop,
            modem: RefCell::new(Some(modem)),
            ready_resp: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl<M> Node for WiFiNode<M>
where
    M: peripheral::Peripheral<P = esp_idf_hal::modem::Modem> + 'static + Send,
{
    fn node_name(&self) -> NodeName {
        NodeName::WiFi
    }

    fn poll(&self, ctx: Rc<dyn Context>, seq: usize) {
        if let Some(m) = self.ready_resp.lock().unwrap().remove(&seq) {
            ctx.boardcast(Message::WiFi(WiFiMessage::ConnectedBoardcast));
            ctx.async_ready(seq, m);
        }
    }

    fn handle_message(&self, _ctx: Rc<dyn Context>, msg: MessageWithHeader) -> HandleResult {
        let seq = msg.seq;
        match msg.body {
            Message::WiFi(msg) => match msg {
                WiFiMessage::ConnectRequest(cfg) => {
                    let modem = self.modem.borrow_mut().take().unwrap();
                    let sysloop = self.sysloop.clone();
                    let ready_resp = self.ready_resp.clone();
                    let nvs = self.nvs.clone();
                    thread::spawn(move || {
                        let mut wifi = EspWifi::new(modem, sysloop.clone(), Some(nvs)).unwrap();

                        let mut wifi = BlockingWifi::wrap(&mut wifi, sysloop.clone()).unwrap();
                        wifi.start().unwrap();

                        info!("Scanning...");
                        let ap_infos = wifi.scan().unwrap();
                        info!("Found {} APs", ap_infos.len());
                        ap_infos.iter().for_each(|ap| info!("AP: {:?}", ap));
                        let w = {
                            let w = ap_infos.iter().find(|ap| ap.ssid == cfg.ssid.as_str());
                            if w.is_none() {
                                panic!("Cannot find AP {}", cfg.ssid)
                            }
                            let w = w.unwrap();
                            info!("Success found AP {:?}", w);
                            if w.auth_method != AuthMethod::None && cfg.password.is_none() {
                                panic!("Missing password for AP {}", cfg.ssid)
                            }
                            w
                        };
                        wifi.set_configuration(&WifiConfiguration::Client(ClientConfiguration {
                            ssid: cfg.ssid.as_str().into(),
                            password: cfg.password.map(|x| x.as_str().into()).unwrap_or_default(),
                            auth_method: w.auth_method,
                            channel: Some(w.channel),
                            ..Default::default()
                        }))
                        .unwrap();
                        wifi.connect().unwrap();

                        info!("Waiting for DHCP lease...");
                        wifi.wait_netif_up().unwrap();

                        let ip_info = wifi.wifi().sta_netif().get_ip_info().unwrap();
                        info!("Wifi DHCP info: {:?}", ip_info);

                        ready_resp
                            .lock()
                            .unwrap()
                            .insert(seq, Message::WiFi(WiFiMessage::ConnectResponse));

                        loop {
                            thread::sleep(Duration::from_secs(1));
                        }
                    });
                    return HandleResult::Pending;
                }
                _ => {}
            },
            _ => {}
        }
        HandleResult::Discard
    }
}

struct TimeNode {
    sntp: RefCell<Option<EspSntp<'static>>>,
}

impl TimeNode {
    pub fn new() -> Self {
        Self {
            sntp: RefCell::new(None),
        }
    }
}

impl Node for TimeNode {
    fn node_name(&self) -> NodeName {
        NodeName::Other("NtpTime".into())
    }

    fn poll(&self, ctx: Rc<dyn Context>, seq: usize) {
        let s = self.sntp.borrow();
        let sntp = s.as_ref().unwrap();
        match sntp.get_sync_status() {
            SyncStatus::Reset => {}
            SyncStatus::Completed => {
                info!("时间同步完成");
                ctx.async_ready(seq, Message::Empty);
            }
            SyncStatus::InProgress => {}
        }
    }

    fn handle_message(&self, ctx: Rc<dyn Context>, msg: MessageWithHeader) -> HandleResult {
        match msg.body {
            Message::WiFi(WiFiMessage::ConnectedBoardcast) => {
                let ntp_server = ipc::StorageClient(ctx)
                    .get("sntp/server".into())
                    .unwrap()
                    .unwrap_or("0.pool.ntp.org".into());
                let sntp = EspSntp::new(&SntpConf {
                    servers: [&ntp_server],
                    sync_mode: SyncMode::Immediate,
                    operating_mode: OperatingMode::Poll,
                })
                .unwrap();
                *self.sntp.borrow_mut() = Some(sntp);
                return HandleResult::Pending;
            }
            _ => {}
        }
        HandleResult::Discard
    }
}

struct HttpClientNodeState {
    // 已经就绪的响应
    ready_resp: HashMap<usize, Message>,
}

pub struct HttpClientNode {
    // 发送一个请求
    req_tx: mpsc::Sender<(usize, HttpRequest)>,
    // 收到一个响应
    resp_rx: mpsc::Receiver<(usize, Message)>,
    state: RefCell<HttpClientNodeState>,
}

impl HttpClientNode {
    pub fn new() -> Self {
        let (req_tx, req_rx) = mpsc::channel::<(usize, HttpRequest)>();
        let (resp_tx, resp_rx) = mpsc::channel();

        thread::spawn(move || {
            let conn = EspHttpConnection::new(&Configuration::default()).unwrap();
            let mut client = Client::wrap(conn);
            loop {
                match req_rx.try_recv() {
                    Ok((seq, req)) => {
                        let req = client.get(&req.url).unwrap().submit().unwrap();
                        let resp_std = ToStd::new(req);
                        let resp_body = resp_std.bytes().map(|x| x.unwrap()).collect::<Vec<_>>();
                        resp_tx
                            .send((
                                seq,
                                Message::Http(HttpMessage::Response(HttpResponse {
                                    body: HttpBody::Bytes(resp_body),
                                })),
                            ))
                            .unwrap();
                    }
                    Err(e) => match e {
                        mpsc::TryRecvError::Empty => {
                            thread::sleep(Duration::from_millis(10));
                        }
                        mpsc::TryRecvError::Disconnected => {
                            return;
                        }
                    },
                }
            }
        });
        Self {
            req_tx,
            resp_rx,
            state: RefCell::new(HttpClientNodeState {
                ready_resp: HashMap::new(),
            }),
        }
    }
}

impl Node for HttpClientNode {
    fn node_name(&self) -> NodeName {
        NodeName::HttpClient
    }

    fn poll(&self, ctx: Rc<dyn Context>, seq: usize) {
        let mut state = self.state.borrow_mut();
        match self.resp_rx.try_recv() {
            Ok((seq, resp)) => {
                // 当消息执行完成后，消息转换为ready态
                state.ready_resp.insert(seq, resp);
            }
            _ => {}
        }
        if state.ready_resp.contains_key(&seq) {
            // 若消息结果为ready态，则返回Sucessful
            let ret = state.ready_resp.remove(&seq).unwrap();
            ctx.async_ready(seq, ret);
        }
    }

    fn handle_message(&self, _ctx: Rc<dyn Context>, msg: MessageWithHeader) -> HandleResult {
        match msg.body {
            Message::Http(HttpMessage::Request(req)) => {
                // 传送消息
                self.req_tx.send((msg.seq, req)).unwrap();
                return HandleResult::Pending;
            }
            _ => {}
        }
        HandleResult::Discard
    }
}

fn main() -> anyhow::Result<()> {
    esp_idf_sys::link_patches();
    esp_idf_svc::log::EspLogger::initialize_default();
    esp_idf_svc::log::set_target_level("esp32c3_impl", log::LevelFilter::Debug)?;
    esp_idf_svc::log::set_target_level("app_core", log::LevelFilter::Debug)?;

    let peripherals = Peripherals::take().unwrap();
    // 所有引脚定义
    let cs = PinDriver::output(peripherals.pins.gpio5)?;
    let dc = PinDriver::output(peripherals.pins.gpio4)?;
    let rst = PinDriver::output(peripherals.pins.gpio8)?;

    // 初始化SPI引脚
    let spi = SpiDeviceDriver::new_single(
        peripherals.spi2,
        peripherals.pins.gpio6,
        peripherals.pins.gpio7,
        Option::<AnyIOPin>::None,
        Option::<AnyIOPin>::None,
        &SpiDriverConfig::default().dma(Dma::Auto(4096)),
        &Config::default()
            .baudrate(80.MHz().into())
            .data_mode(MODE_3),
    )?;

    // 设置底部灯为关闭
    let mut blue_led = LedcDriver::new(
        peripherals.ledc.channel0,
        LedcTimerDriver::new(
            peripherals.ledc.timer0,
            &TimerConfig::new().frequency(25.kHz().into()),
        )
        .unwrap(),
        peripherals.pins.gpio2,
    )
    .unwrap();
    blue_led.set_duty(0).unwrap();

    // 设置屏幕背光亮度为100%
    let mut screen_ledc = LedcDriver::new(
        peripherals.ledc.channel1,
        LedcTimerDriver::new(
            peripherals.ledc.timer1,
            &TimerConfig::new().frequency(25.kHz().into()),
        )
        .unwrap(),
        peripherals.pins.gpio10,
    )
    .unwrap();
    screen_ledc.set_duty(screen_ledc.get_max_duty()).unwrap();

    // 初始化显示屏驱动
    let display = Builder::st7789(SPIInterface::new(spi, dc, cs))
        .with_display_size(240, 240)
        .with_framebuffer_size(240, 240)
        .with_orientation(Orientation::Portrait(false))
        .with_invert_colors(ColorInversion::Inverted)
        .init(&mut FreeRtos, Some(rst))
        .unwrap();

    let nvs = EspDefaultNvsPartition::take()?;

    let platform = embedded_software_slint_backend::MySoftwarePlatform::new(
        Rc::new(RefCell::new(display)),
        Some(|_| Ok(())),
    );
    slint::platform::set_platform(Box::new(platform)).map_err(|e| anyhow::anyhow!("{:?}", e))?;

    let btn_pin = PinDriver::input(peripherals.pins.gpio9)?;

    let one_butten_node = EspOneButton::new(btn_pin);
    let sche = get_scheduler();
    sche.register_node(one_butten_node);
    sche.register_node(PerformanceNode {});
    sche.register_node(WiFiNode::new(
        nvs.clone(),
        EspSystemEventLoop::take().unwrap(),
        peripherals.modem,
    ));
    sche.register_node(TimeNode::new());
    sche.register_node(HttpClientNode::new());
    let sche_timer = slint::Timer::default();
    sche_timer.start(
        slint::TimerMode::Repeated,
        Duration::from_millis(20),
        move || {
            sche.schedule_once();
        },
    );

    slint::run_event_loop().map_err(|e| anyhow::anyhow!("{:?}", e))?;
    Ok(())
}
