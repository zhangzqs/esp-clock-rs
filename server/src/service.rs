mod openwrt;
pub use openwrt::{OpenWrt, OpenWrtServiceConfig};

mod ping;
pub use ping::PingService;

mod photo;
pub use photo::PhotoService;

mod weather;
pub use weather::{WeatherService, WeatherServiceConfig};