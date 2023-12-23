use slint_app::System;

#[derive(Clone, Copy)]
pub struct EspSystem;

unsafe impl Send for EspSystem {}
unsafe impl Sync for EspSystem {}

impl System for EspSystem {
    /// 重启
    fn restart(&self) {
        unsafe {
            esp_idf_sys::esp_restart();
        }
    }

    /// 获取剩余可用堆内存，这可能比最大连续的可分配块的值还要大
    fn get_free_heap_size(&self) -> usize {
        unsafe { esp_idf_sys::esp_get_free_heap_size() as usize }
    }

    /// 获取最大连续的可分配块
    fn get_largest_free_block(&self) -> usize {
        unsafe { esp_idf_sys::heap_caps_get_largest_free_block(esp_idf_sys::MALLOC_CAP_8BIT) }
    }

    // fn get_sta_netif(&self) -> Option<embedded_svc::ipv4::IpInfo> {
    //     let wifi = self.wifi.lock().unwrap();
    //     if wifi.is_none() {
    //         return None;
    //     }
    //     let wifi = wifi.as_ref().unwrap();
    //     let netif = wifi.sta_netif();
    //     let ip_info = netif.get_ip_info().unwrap();
    //     Some(ip_info)
    // }
}
