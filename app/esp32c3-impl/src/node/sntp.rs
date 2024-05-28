use std::{cell::RefCell, rc::Rc};

use app_core::proto::*;
use esp_idf_svc::sntp::{EspSntp, OperatingMode, SntpConf, SyncMode, SyncStatus};
use esp_idf_sys as _;
use log::info;

pub struct SntpService {
    sntp: RefCell<Option<EspSntp<'static>>>,
}

impl SntpService {
    pub fn new() -> Self {
        Self {
            sntp: RefCell::new(None),
        }
    }
}

impl Node for SntpService {
    fn node_name(&self) -> NodeName {
        NodeName::Other("EspSntpTime".into())
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
        if let Message::WiFi(WiFiMessage::ConnectedBoardcast) = msg.body {
            let ntp_server = ipc::StorageClient(ctx)
                .get("sntp/server".into())
                .unwrap()
                .as_str("")
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
        HandleResult::Discard
    }
}
