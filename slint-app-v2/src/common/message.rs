mod scheduler;
pub use scheduler::SchedulerMessage;

#[derive(Debug, Clone, Copy)]
pub enum Message {
    Empty,
    SchedulerMessage(SchedulerMessage),
    HomeMessage,
    WeatherMessage,
}
