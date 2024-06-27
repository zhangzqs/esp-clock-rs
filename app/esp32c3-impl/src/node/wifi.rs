use std::{
    collections::HashMap,
    rc::Rc,
    str::FromStr,
    sync::{
        mpsc::{channel, Receiver, Sender},
        Arc, Mutex,
    },
    thread,
};

use app_core::proto::*;
use embedded_svc::wifi::Wifi;
use esp_idf_hal::peripheral;
use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    nvs::EspDefaultNvsPartition,
    wifi::{
        AccessPointConfiguration, AuthMethod, BlockingWifi, ClientConfiguration, Configuration,
        EspWifi,
    },
};
use log::{error, info};

pub struct WiFiService {
    ready_resp: Arc<Mutex<HashMap<usize, WiFiMessage>>>,
    msg_sender: Sender<(usize, WiFiMessage)>,
}

enum WiFiMode {
    None,
    STA,
    AP,
}

impl WiFiService {
    pub fn new(
        nvs: EspDefaultNvsPartition,
        sysloop: EspSystemEventLoop,
        modem: impl peripheral::Peripheral<P = esp_idf_hal::modem::Modem> + 'static + Send,
    ) -> Self {
        let ready_resp = Arc::new(Mutex::new(HashMap::new()));
        let (tx, rx) = channel();
        Self::spawn_forever(nvs, sysloop, modem, ready_resp.clone(), rx);
        Self {
            ready_resp: ready_resp.clone(),
            msg_sender: tx,
        }
    }

    fn handle_sta(
        cfg: WiFiStorageConfiguration,
        wifi: &mut BlockingWifi<&mut EspWifi>,
    ) -> Result<(), WiFiError> {
        wifi.set_configuration(&Configuration::Client(ClientConfiguration::default()))
            .map_err(|x| WiFiError::Other(format!("{x:?}")))?;
        wifi.start()
            .map_err(|x| WiFiError::Other(format!("{x:?}")))?;

        info!("Scanning...");
        let ap_infos = wifi
            .scan()
            .map_err(|x| WiFiError::Other(format!("{x:?}")))?;
        info!("Found {} APs", ap_infos.len());
        ap_infos.iter().for_each(|ap| info!("AP: {:?}", ap));
        let w = match ap_infos.iter().find(|ap| ap.ssid == cfg.ssid.as_str()) {
            Some(x) => {
                info!("Success found AP {:?}", x);
                if x.auth_method.is_some()
                    && x.auth_method.unwrap() != AuthMethod::None
                    && cfg.password.is_none()
                {
                    error!("Missing password for AP {}", cfg.ssid);
                    return Err(WiFiError::ApNeedPassword);
                }
                x
            }
            None => {
                error!("Not Found AP");
                return Err(WiFiError::NotFoundAP);
            }
        };

        wifi.set_configuration(&Configuration::Client(ClientConfiguration {
            ssid: w.ssid.clone(),
            password: heapless::String::<64>::from_str(&cfg.password.unwrap_or_default()).unwrap(),
            auth_method: w.auth_method.unwrap_or(AuthMethod::None),
            channel: Some(w.channel),
            ..Default::default()
        }))
        .map_err(|x| WiFiError::Other(format!("{x:?}")))?;
        wifi.connect()
            .map_err(|x| WiFiError::Other(format!("{x:?}")))?;

        info!("Waiting for DHCP lease...");
        wifi.wait_netif_up()
            .map_err(|x| WiFiError::Other(format!("{x:?}")))?;

        let ip_info = wifi
            .wifi()
            .sta_netif()
            .get_ip_info()
            .map_err(|x| WiFiError::Other(format!("{x:?}")))?;
        info!("Wifi DHCP info: {:?}", ip_info);
        Ok(())
    }

    fn handle_ap(wifi: &mut BlockingWifi<&mut EspWifi>) -> Result<(), WiFiError> {
        wifi.set_configuration(&Configuration::AccessPoint(AccessPointConfiguration {
            ssid: "ESP-CLOCK-RS".try_into().unwrap(),
            ..Default::default()
        }))
        .map_err(|x| WiFiError::Other(format!("{x:?}")))?;
        wifi.start()
            .map_err(|x| WiFiError::Other(format!("{x:?}")))?;

        info!("Waiting for DHCP lease...");
        wifi.wait_netif_up()
            .map_err(|x| WiFiError::Other(format!("{x:?}")))?;

        let ip_info = wifi
            .wifi()
            .ap_netif()
            .get_ip_info()
            .map_err(|x| WiFiError::Other(format!("{x:?}")))?;
        info!("Wifi DHCP info: {:?}", ip_info);
        Ok(())
    }

    fn handle_ip_info(
        wifi: &mut BlockingWifi<&mut EspWifi>,
        mode: &WiFiMode,
    ) -> Result<NetIpInfo, WiFiError> {
        match mode {
            WiFiMode::None => Err(WiFiError::NotStarted),
            WiFiMode::STA => {
                let info = wifi
                    .wifi()
                    .sta_netif()
                    .get_ip_info()
                    .map_err(|x| WiFiError::Other(format!("{x:?}")))?;
                Ok(NetIpInfo { ip: info.ip })
            }
            WiFiMode::AP => {
                let info = wifi
                    .wifi()
                    .ap_netif()
                    .get_ip_info()
                    .map_err(|x| WiFiError::Other(format!("{x:?}")))?;
                Ok(NetIpInfo { ip: info.ip })
            }
        }
    }

    fn spawn_forever(
        nvs: EspDefaultNvsPartition,
        sysloop: EspSystemEventLoop,
        modem: impl peripheral::Peripheral<P = esp_idf_hal::modem::Modem> + 'static + Send,
        ready_resp: Arc<Mutex<HashMap<usize, WiFiMessage>>>,
        msg_receiver: Receiver<(usize, WiFiMessage)>,
    ) {
        thread::Builder::new()
            .stack_size(8192)
            .spawn(move || {
                let mut wifi = EspWifi::new(modem, sysloop.clone(), Some(nvs)).unwrap();
                let mut wifi = BlockingWifi::wrap(&mut wifi, sysloop.clone()).unwrap();

                let mut mode = WiFiMode::None;
                for (seq, msg) in msg_receiver.iter() {
                    let r = || -> Result<WiFiMessage, WiFiError> {
                        match msg {
                            WiFiMessage::ConnectRequest(cfg) => {
                                Self::handle_sta(cfg, &mut wifi)?;
                                mode = WiFiMode::STA;
                                Ok(WiFiMessage::ConnectResponse)
                            }
                            WiFiMessage::StartAPRequest => {
                                Self::handle_ap(&mut wifi)?;
                                mode = WiFiMode::AP;
                                Ok(WiFiMessage::StartAPResponse)
                            }
                            WiFiMessage::GetIpInfoRequest => Ok(WiFiMessage::GetIpInfoResponse(
                                Self::handle_ip_info(&mut wifi, &mode)?,
                            )),
                            m => panic!("unsupported message {m:?}"),
                        }
                    }();
                    ready_resp.lock().unwrap().insert(
                        seq,
                        match r {
                            Ok(x) => x,
                            Err(e) => WiFiMessage::Error(e),
                        },
                    );
                }
            })
            .unwrap();
    }
}

impl Node for WiFiService {
    fn node_name(&self) -> NodeName {
        NodeName::WiFi
    }

    fn poll(&self, ctx: Rc<dyn Context>, seq: usize) {
        if let Some(m) = self.ready_resp.lock().unwrap().remove(&seq) {
            match m {
                WiFiMessage::ConnectResponse => {
                    ctx.broadcast_topic(
                        TopicName::WiFi,
                        Message::WiFi(WiFiMessage::ConnectedBroadcast),
                    );
                }
                WiFiMessage::StartAPResponse => {
                    ctx.broadcast_topic(
                        TopicName::WiFi,
                        Message::WiFi(WiFiMessage::APStartedBroadcast),
                    );
                }
                _ => {}
            }
            ctx.async_ready(seq, Message::WiFi(m));
        }
    }

    fn handle_message(&self, _ctx: Rc<dyn Context>, msg: MessageWithHeader) -> HandleResult {
        let seq = msg.seq;

        // 屏蔽自己的广播消息
        if let Message::WiFi(msg) = msg.body {
            match msg {
                WiFiMessage::ConnectRequest(_)
                | WiFiMessage::StartAPRequest
                | WiFiMessage::GetIpInfoRequest => {
                    // 由于确保了wifi thread常驻后台，不会结束，故此处可以直接unwrap
                    self.msg_sender.send((seq, msg)).unwrap();
                    return HandleResult::Pending;
                }
                _ => {}
            }
        }

        HandleResult::Discard
    }
}
