use std::sync::{Mutex, Arc};

use esp_idf_svc::wifi::EspWifi;
use slint_app::NetworkState;

struct EspNetworkState<'a> {
    wifi: Arc<Mutex<Option<EspWifi<'a>>>>,
}
impl NetworkState for EspNetworkState<'_> {
    fn get_sta_netif(&self) -> Option<embedded_svc::ipv4::IpInfo> {
        let wifi = self.wifi.lock().unwrap();
        if wifi.is_none() {
            return None;
        }
        let wifi = wifi.as_ref().unwrap();
        let netif = wifi.sta_netif();
        let ip_info = netif.get_ip_info().unwrap();
        Some(ip_info)
    }
}
