mod home;
mod weather;

pub use {home::HomeApp, weather::WeatherApp};
slint::include_modules!();
