use core::ops::Sub;
use core::time::Duration;

pub trait TimeManagerInterface {
    /// a monotonically increasing clock.
    fn now(&self) -> Duration;

    /// how much time passed since time_in_the_past.
    fn since(&self, time_in_the_past: Duration) -> Duration {
        self.now().sub(time_in_the_past)
    }
}

#[cfg(feature = "std")]
pub use std_time::*;

#[cfg(feature = "std")]
mod std_time {
    use crate::TimeManagerInterface;
    use std::ops::Sub;
    use std::time::{Duration, Instant, SystemTime};

    pub struct TimeManager;

    impl TimeManager {
        pub fn new() -> Self {
            TimeManager
        }
    }
    
    impl TimeManagerInterface for TimeManager {
        /// Duration from EPOCH.
        fn now(&self) -> Duration {
            SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .expect("Failed to get current time")
        }
    }
}
