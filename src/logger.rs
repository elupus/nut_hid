use log::*;
use wdk::println;

pub struct WdkLogger;

impl log::Log for WdkLogger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            println!("{} - {}", record.level(), record.args());
        }
    }

    fn flush(&self) {}
}

impl WdkLogger {
    pub fn init() {
        static LOGGER: WdkLogger = WdkLogger;

        log::set_max_level(LevelFilter::Debug);
        match log::set_logger(&LOGGER) {
            Err(err) => {
                println!("Failed to init logger: {:?}", err);
            }
            Ok(_) => (),
        }
    }
}
