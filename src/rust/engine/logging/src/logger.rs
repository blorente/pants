// Copyright 2018 Pants project contributors (see CONTRIBUTORS.md).
// Licensed under the Apache License, Version 2.0 (see LICENSE).

use crate::TryIntoPythonLogLevel;
use lazy_static::lazy_static;
use log::{log, set_logger, set_max_level, LevelFilter, Log, Metadata, Record};
use parking_lot::Mutex;
use simplelog::Config;
use simplelog::WriteLogger;
use std::fs::File;
use std::fs::OpenOptions;
use std::io::{stderr, Stderr, Write};
use std::path::PathBuf;

lazy_static! {
  pub static ref LOGGER: Logger = Logger::new();
}

pub struct Logger {
  pantsd_log: Mutex<MaybeWriteLogger<File>>,
  stderr_log: Mutex<MaybeWriteLogger<Stderr>>,
}

impl Logger {
  pub fn new() -> Logger {
    Logger {
      pantsd_log: Mutex::new(MaybeWriteLogger::empty()),
      stderr_log: Mutex::new(MaybeWriteLogger::empty()),
    }
  }

  // TODO Maybe return a Result<(), String> which we pass to Python as a PyResult.
  // This is not possible atm because this function gets called before the externs
  // are initialized.
  pub fn init(max_level: u64) {
    let max_python_level = (max_level).try_into_PythonLogLevel();
    match max_python_level {
      Ok(python_level) => {
        let level: log::LevelFilter = python_level.into();
        set_max_level(level);
        set_logger(&*LOGGER).expect("Error setting up global logger.");
      }
      Err(err) => panic!("Unrecognised log level from python: {}: {}", max_level, err),
    };
  }

  pub fn set_max_level_from_python(&self, python_level: u64) -> Result<(), String> {
    python_level
      .try_into_PythonLogLevel()
      .map(|level| set_max_level(level.into()))
  }

  pub fn set_stderr_logger(&self, python_level: u64) -> Result<(), String> {
    python_level
      .try_into_PythonLogLevel()
      .map(|level| *self.stderr_log.lock() = MaybeWriteLogger::new(stderr(), level.into()))
  }

  pub fn set_pantsd_logger(&self, log_file_path: PathBuf, python_level: u64) -> Result<(), String> {
    python_level.try_into_PythonLogLevel().and_then(|level| {
      OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_file_path)
        .map(|file| {
          *self.pantsd_log.lock() = MaybeWriteLogger::new(file, level.into());
        })
        .map_err(|err| format!("Error opening pantsd logfile: {}", err))
    })
  }

  pub fn log_from_python(
    &self,
    message: &str,
    python_level: u64,
    target: &str,
  ) -> Result<(), String> {
    let a = OpenOptions::new()
        .create(true)
        .append(true)
        .open(PathBuf::from("/tmp/rlog"))
        .map(|mut file|
            file.write(format!("BL: log_from_python: {:?}, level {:?}, max_level {:?}\n", message, python_level, log::max_level()).as_ref()).expect("error")).expect("error");
    python_level.try_into_PythonLogLevel().map(|level| {
      log!(target: target, level.into(), "{}", message);
    })
  }
}

impl Log for Logger {
  fn enabled(&self, _metadata: &Metadata) -> bool {
    // Individual log levels are handled by each sub-logger,
    // And a global filter is applied to set_max_level.
    // No need to filter here.
    true
  }

  fn log(&self, record: &Record) {
    self.stderr_log.lock().log(record);
    self.pantsd_log.lock().log(record);
    let a = OpenOptions::new()
        .create(true)
        .append(true)
        .open(PathBuf::from("/tmp/rlog"))
        .map(|mut file|
            file.write(format!("BL: Logger::log() with level {:?}, msg: {:?}\n", record.level(), record.args()).as_ref()).expect("error")).expect("error");
  }

  fn flush(&self) {
    self.stderr_log.lock().flush();
    self.pantsd_log.lock().flush();
  }
}

struct MaybeWriteLogger<W: Write + Send + 'static> {
  level: LevelFilter,
  inner: Option<Box<WriteLogger<W>>>,
}

impl<W: Write + Send + 'static> MaybeWriteLogger<W> {
  pub fn empty() -> MaybeWriteLogger<W> {
    MaybeWriteLogger {
      level: LevelFilter::Off,
      inner: None,
    }
  }

  pub fn new(writable: W, level: LevelFilter) -> MaybeWriteLogger<W> {
    // We initialize the inner WriteLogger with no filters so that we don't
    // have to create a new one every time we change the level of the outer
    // MaybeWriteLogger.
    MaybeWriteLogger {
      level,
      inner: Some(WriteLogger::new(
        LevelFilter::max(),
        Config::default(),
        writable,
      )),
    }
  }

  pub fn level(&self) -> LevelFilter {
    self.level
  }
}

impl<W: Write + Send + 'static> Log for MaybeWriteLogger<W> {
  fn enabled(&self, metadata: &Metadata) -> bool {
    metadata.level() <= self.level()
  }

  fn log(&self, record: &Record) {
    if !self.enabled(record.metadata()) {
      return;
    }
    if let Some(ref logger) = self.inner {
      let a = OpenOptions::new()
          .create(true)
          .append(true)
          .open(PathBuf::from("/tmp/rlog"))
          .map(|mut file|
              file.write(format!("BL: logging with level {:?}, msg: {:?}\n", record.level(), record.args()).as_ref()).expect("error")).expect("error");
      logger.log(record);
    } else {
      let a = OpenOptions::new()
          .create(true)
          .append(true)
          .open(PathBuf::from("/tmp/rlog"))
          .map(|mut file|
              file.write(format!("BL: failed to log with level {:?}, msg: {:?}\n", record.level(), record.args()).as_ref()).expect("error")).expect("error");
    }
  }

  fn flush(&self) {
    if let Some(ref logger) = self.inner {
      logger.flush();
    }
  }
}
