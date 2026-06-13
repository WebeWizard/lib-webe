use webe_log::{ConsoleLogger, LogLevel, WebeLogger};

#[test]
fn console_log_test() {
    let mut logger = WebeLogger::new();
    logger.add_sink(Box::new(ConsoleLogger::new(logger.mon_sender.clone())));
    logger.log(&LogLevel::INFO, "hi mom");
    // sleep the main thread so the sink's can write on their own schedules
    std::thread::sleep(std::time::Duration::from_secs(5));
}
