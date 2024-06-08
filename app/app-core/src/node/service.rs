mod midiplayer;
mod onebutton;
mod system;
mod router;
mod storage;
mod timer;
mod weather;
mod wifi;

pub use {
    midiplayer::MidiPlayerService, onebutton::TouchOneButtonAdapterService,
    system::MockSystemService, router::RouterService, storage::MockStorageService,
    timer::TimerService, weather::WeatherService, wifi::MockWiFiService,
};
