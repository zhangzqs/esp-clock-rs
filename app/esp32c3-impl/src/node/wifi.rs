use std::{
    cell::RefCell,
    collections::HashMap,
    rc::Rc,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

use app_core::proto::*;
use esp_idf_hal::peripheral;
use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    nvs::EspDefaultNvsPartition,
    wifi::{AuthMethod, BlockingWifi, ClientConfiguration, Configuration, EspWifi},
};
use esp_idf_sys as _;
use log::info;

pub struct WiFiService<M> {
    sysloop: EspSystemEventLoop,
    modem: RefCell<Option<M>>,
    ready_resp: Arc<Mutex<HashMap<usize, Message>>>,
    nvs: EspDefaultNvsPartition,
}

impl<M> WiFiService<M> {
    pub fn new(nvs: EspDefaultNvsPartition, sysloop: EspSystemEventLoop, modem: M) -> Self {
        Self {
            nvs,
            sysloop,
            modem: RefCell::new(Some(modem)),
            ready_resp: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl<M> Node for WiFiService<M>
where
    M: peripheral::Peripheral<P = esp_idf_hal::modem::Modem> + 'static + Send,
{
    fn node_name(&self) -> NodeName {
        NodeName::WiFi
    }

    fn poll(&self, ctx: Rc<dyn Context>, seq: usize) {
        if let Some(m) = self.ready_resp.lock().unwrap().remove(&seq) {
            ctx.broadcast_global(Message::WiFi(WiFiMessage::ConnectedBoardcast));
            ctx.async_ready(seq, m);
        }
    }

    fn handle_message(&self, _ctx: Rc<dyn Context>, msg: MessageWithHeader) -> HandleResult {
        let seq = msg.seq;
        if let Message::WiFi(msg) = msg.body {
            if let WiFiMessage::ConnectRequest(cfg) = msg {
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
                    wifi.set_configuration(&Configuration::Client(ClientConfiguration {
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
        }
        HandleResult::Discard
    }
}
