use log::{set_boxed_logger, set_max_level, Log};
use parking_lot::Mutex;
use std::collections::VecDeque;
use ui::display::EngineDisplay;

pub struct UILogger {
  level_filter: log::LevelFilter,
  logs: Mutex<VecDeque<String>>,
  display: Option<EngineDisplay>,
}

impl UILogger {
  pub fn new() -> UILogger {
    UILogger {
      level_filter: log::LevelFilter::Debug,
      logs: Mutex::new(VecDeque::with_capacity(500)),
      display: EngineDisplay::create(4, true),
    }
  }

  pub fn init() {
    set_max_level(log::LevelFilter::Debug);
    set_boxed_logger(Box::new(UILogger::new()))
      .expect("Failed to set logger (maybe you tried to call init multiple times?)");
  }

  //    pub fn create_display(&mut self) -> Option<EngineDisplay> {
  //        let mut display = EngineDisplay::create(4, true).unwrap();
  //        for message in self.logs {
  //            display.log(message.clone())
  //        }
  //        display
  //    }
}

impl Log for UILogger {
  fn enabled(&self, metadata: &log::Metadata) -> bool {
    metadata.level() <= self.level_filter
  }

  fn log(&self, record: &log::Record) {
    if !self.enabled(record.metadata()) {
      return;
    }
    let message = format!("{}", record.args());
    println!("BL: Logging {}", message);
    //        self.display.log(message);
  }

  fn flush(&self) {}
}
