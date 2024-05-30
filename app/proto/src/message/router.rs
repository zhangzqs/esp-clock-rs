use crate::NodeName;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RoutePage {
    Boot,
    Home,
    Menu,
    Weather,
}

impl RoutePage {
    pub fn map_to_node_name(&self) -> NodeName {
        match *self {
            RoutePage::Boot => NodeName::BootPage,
            RoutePage::Home => NodeName::HomePage,
            RoutePage::Menu => NodeName::MenuPage,
            RoutePage::Weather => NodeName::WeatherPage,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RouterMessage {
    GotoPage(RoutePage),
}
