mod home;
mod menu;
mod weather;

pub use {home::HomeApp, menu::MenuApp, weather::WeatherApp};
slint::include_modules!();
