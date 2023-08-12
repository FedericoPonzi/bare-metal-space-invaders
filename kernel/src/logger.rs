use crate::println;
use log::{LevelFilter, Log, Metadata, Record, SetLoggerError};

/// The IrisOS's logger.
/// It's not using allocations, if you plan to change it to do allocations you might want
/// to ignore prints from allocator modules.
pub struct IrisLogger {
    /// The default logging level
    default_level: LevelFilter,

    /// The specific logging level for each module
    ///
    /// This is used to override the default value for some specific modules.
    /// After initialization, the vector is sorted so that the first (prefix) match
    /// directly gives us the desired log level.
    module_levels: [(&'static str, LevelFilter); 1],
}
impl IrisLogger {
    pub const fn new() -> IrisLogger {
        Self {
            default_level: LevelFilter::Trace,
            module_levels: [
                // Want to filter out modules? Add them here!
                //("iris_os::allocator2", LevelFilter::Off),
                ("crate::mailbox", LevelFilter::Off),
            ],
        }
    }

    pub fn init(&'static self) -> Result<(), SetLoggerError> {
        log::set_max_level(self.default_level);
        unsafe { log::set_logger_racy(self) }
    }
}
impl Log for IrisLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        &metadata.level().to_level_filter()
            <= self
                .module_levels
                .iter()
                .find(|(name, _level)| metadata.target().starts_with(name))
                .map(|(_name, level)| level)
                .unwrap_or(&self.default_level)
    }

    fn log(&self, record: &Record) {
        if !self.enabled(record.metadata()) {
            return;
        }
        println!("[{}] {}", record.level(), record.args());
    }
    fn flush(&self) {}
}
