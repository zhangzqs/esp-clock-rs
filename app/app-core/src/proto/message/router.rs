use crate::proto::NodeName;

#[derive(Debug, Clone)]
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

#[derive(Debug, Clone)]
pub enum RouterMessage {
    GotoPage(RoutePage),
}
