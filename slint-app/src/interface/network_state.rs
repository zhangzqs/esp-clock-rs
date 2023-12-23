pub trait NetworkState: Send + Sync {
    fn get_sta_netif(&self) -> Option<embedded_svc::ipv4::IpInfo>;
}