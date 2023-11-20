use anyhow::{bail, Result};
use embedded_svc::wifi::{
    AuthMethod, ClientConfiguration, Configuration,
};
use esp_idf_hal::peripheral;
use esp_idf_svc::{
    eventloop::EspSystemEventLoop,
    nvs::{EspNvsPartition, NvsDefault},
    wifi::BlockingWifi,
    wifi::EspWifi,
};
use log::info;

pub fn connect_to_wifi(
    ssid: &str,
    pass: &str,
    modem: impl peripheral::Peripheral<P = esp_idf_hal::modem::Modem> + 'static,
    sysloop: EspSystemEventLoop,
    nvs: Option<EspNvsPartition<NvsDefault>>,
) -> Result<Box<EspWifi<'static>>> {
    if ssid.is_empty() {
        bail!("Missing WiFi name")
    }

    let mut esp_wifi = EspWifi::new(modem, sysloop.clone(), nvs)?;
    let mut wifi = BlockingWifi::wrap(&mut esp_wifi, sysloop)?;
    info!("Starting wifi...");
    wifi.start()?;

    info!("Scanning...");
    let ap_infos = wifi.scan()?;
    info!("Found {} APs", ap_infos.len());
    ap_infos.iter().for_each(|ap| info!("AP: {:?}", ap));
    let w = {
        let w = ap_infos.iter().find(|ap| ap.ssid == ssid);
        if w.is_none() {
            bail!("Cannot find AP {}", ssid)
        }
        let w = w.unwrap();
        info!("Success found AP {:?}", w);
        if w.auth_method != AuthMethod::None && pass.is_empty() {
            bail!("Missing password for AP {}", ssid)
        }
        w
    };
    wifi.set_configuration(&Configuration::Client(ClientConfiguration {
        ssid: w.ssid.clone(),
        password: pass.into(),
        auth_method: w.auth_method,
        channel: Some(w.channel),
        ..Default::default()
    }))?;

    info!("Connecting wifi...");
    for i in 0..3 {
        if wifi.connect().is_ok() {
            info!("Wifi Connected!");
            break;
        } else {
            info!("Attempt {}/3...", i + 1);
        }
    }

    info!("Waiting for DHCP lease...");
    wifi.wait_netif_up()?;

    let ip_info = wifi.wifi().sta_netif().get_ip_info()?;
    info!("Wifi DHCP info: {:?}", ip_info);

    Ok(Box::new(esp_wifi))
}
