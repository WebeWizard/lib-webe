
use webe_log::{WebeLogger,ConsoleLogger,LogLevel};

#[test]
fn console_log_test() {
  let mut webeLogger = WebeLogger::new();
  webeLogger.add_sink(
    Box::new(ConsoleLogger::new())
  );
  webeLogger.log(&LogLevel::INFO, "hi mom");
}