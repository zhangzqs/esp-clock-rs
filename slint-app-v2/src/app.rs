mod home;
mod menu;
mod router;
mod weather;

pub use {home::HomePageApp, menu::MenuPageApp, router::RouterApp, weather::WeatherPageApp};
slint::include_modules!();
