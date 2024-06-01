use crate::{proto::RoutePage, ui::PageRouteTable};

pub fn proto_route_table_to_slint_route_table(r: RoutePage) -> PageRouteTable {
    match r {
        RoutePage::Boot => PageRouteTable::Boot,
        RoutePage::Home => PageRouteTable::Home,
        RoutePage::Menu => PageRouteTable::Menu,
        RoutePage::Weather => PageRouteTable::Weather,
        RoutePage::Music => PageRouteTable::Music,
    }
}

pub fn slint_route_table_to_proto_route_table(r: PageRouteTable) -> RoutePage {
    match r {
        PageRouteTable::Boot => RoutePage::Boot,
        PageRouteTable::Home => RoutePage::Home,
        PageRouteTable::Menu => RoutePage::Menu,
        PageRouteTable::Weather => RoutePage::Weather,
        PageRouteTable::Music => RoutePage::Music,
    }
}
