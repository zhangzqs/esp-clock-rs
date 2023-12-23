mod projector;
pub use projector::ProjectorApp;

mod photo;
pub use photo::PhotoApp;

mod clock;
pub use clock::ClockApp;

mod fpstest;
pub use fpstest::FPSTestApp;

mod evil_apple;
pub use evil_apple::{EvilApple, EvilAppleApp};

mod music;
pub use music::MusicApp;

mod home;
pub use home::HomeApp;

mod network;
pub use network::NetworkMonitorApp;

mod server;
pub use server::HttpServerApp;