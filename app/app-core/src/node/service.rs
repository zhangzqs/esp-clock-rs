mod midiplayer;
mod onebutton;
mod router;
mod storage;
mod system;
mod timer;
mod weather;
mod wifi;

pub use {
    midiplayer::MidiPlayerService, onebutton::TouchOneButtonAdapterService, router::RouterService,
    storage::MockStorageService, system::MockSystemService, timer::TimerService,
    weather::WeatherService, wifi::MockWiFiService,
};
