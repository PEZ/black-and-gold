use log::Log;
use log::{Level, LevelFilter, Metadata, Record};
#[cfg(target_os = "ios")]
use oslog::OsLogger;

pub struct SimpleLogger;

impl Log for SimpleLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Info
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            {
                println!("{} - {}", record.level(), record.args());
            }
        }
    }

    fn flush(&self) {}
}

pub fn setup_logger() {
    #[cfg(not(target_os = "ios"))]
    log::set_boxed_logger(Box::new(SimpleLogger))
        .map(|()| log::set_max_level(LevelFilter::Info))
        .expect("Failed to set logger");

    #[cfg(target_os = "ios")]
    {
        OsLogger::new("news.bg.BG")
            .level_filter(LevelFilter::Debug)
            .init()
            .unwrap();
    }
}
