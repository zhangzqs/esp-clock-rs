mod scheduler;
pub use scheduler::SchedulerMessage;

#[derive(Debug, Clone, Copy)]
pub enum Message {
    SchedulerMessage(SchedulerMessage),
    HomeMessage,
    WeatherMessage,
}
