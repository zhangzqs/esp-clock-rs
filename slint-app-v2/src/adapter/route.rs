use crate::ui;
use proto;

pub fn proto_route_table_to_slint_route_table(r: proto::RoutePage) -> ui::PageRouteTable {
    match r {
        proto::RoutePage::Boot => ui::PageRouteTable::Boot,
        proto::RoutePage::Home => ui::PageRouteTable::Home,
        proto::RoutePage::Menu => ui::PageRouteTable::Menu,
        proto::RoutePage::Weather => ui::PageRouteTable::Weather,
    }
}

pub fn slint_route_table_to_proto_route_table(r: ui::PageRouteTable) -> proto::RoutePage {
    match r {
        ui::PageRouteTable::Boot => proto::RoutePage::Boot,
        ui::PageRouteTable::Home => proto::RoutePage::Home,
        ui::PageRouteTable::Menu => proto::RoutePage::Menu,
        ui::PageRouteTable::Weather => proto::RoutePage::Weather,
    }
}
