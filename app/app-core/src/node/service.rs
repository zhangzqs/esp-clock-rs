mod midiplayer;
mod onebutton;
mod performance;
mod router;
mod storage;
mod timer;
mod timestamp;
mod weather;
mod wifi;

pub use {
    midiplayer::MidiPlayerService, onebutton::TouchOneButtonAdapterService,
    performance::MockPerformanceService, router::RouterService, storage::MockStorageService,
    timer::TimerService, timestamp::DefaultTimestampService, weather::WeatherService,
    wifi::MockWiFiService,
};
