mod home;
mod scheduler;
pub use {home::HomeMessage, scheduler::SchedulerMessage};

use crate::app::PageRouteTable;

#[derive(Debug, Clone)]
pub enum Message {
    Empty,
    Scheduler(SchedulerMessage),
    GoToPage(PageRouteTable),
    HomePage(HomeMessage),
    Weather,
}
