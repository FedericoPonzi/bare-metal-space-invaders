use std::time::Duration;

pub trait TimeManagerInterface {
    fn wait(&self, d: Duration);
}

pub fn time_manager() -> TimeManager {
    TimeManager {}
}

pub struct TimeManager {}

impl TimeManagerInterface for TimeManager {
    fn wait(&self, d: Duration) {
        std::thread::sleep(d);
    }
}
