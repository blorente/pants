// Enable all clippy lints except for many of the pedantic ones. It's a shame this needs to be copied and pasted across crates, but there doesn't appear to be a way to include inner attributes from a common source.
#![cfg_attr(
  feature = "cargo-clippy",
  deny(
    clippy,
    default_trait_access,
    expl_impl_clone_on_copy,
    if_not_else,
    needless_continue,
    single_match_else,
    unseparated_literal_suffix,
    used_underscore_binding
  )
)]
// It is often more clear to show that nothing is being moved.
#![cfg_attr(feature = "cargo-clippy", allow(match_ref_pats))]
// Subjective style.
#![cfg_attr(
  feature = "cargo-clippy",
  allow(len_without_is_empty, redundant_field_names)
)]
// Default isn't as big a deal as people seem to think it is.
#![cfg_attr(
  feature = "cargo-clippy",
  allow(new_without_default, new_without_default_derive)
)]
// Arc<Mutex> can be more clear than needing to grok Orderings:
#![cfg_attr(feature = "cargo-clippy", allow(mutex_atomic))]

extern crate lazy_static;
extern crate log;
extern crate num_enum;
extern crate parking_lot;
extern crate rand;
extern crate termion;
extern crate unicode_segmentation;

pub mod display;

use num_enum::CustomTryInto;

// This is a hard-coding of constants in the standard logging python package.
// TODO: Switch from CustomTryInto to TryFromPrimitive when try_from is stable.
#[derive(Debug, Eq, PartialEq, CustomTryInto)]
#[repr(u8)]
pub enum PythonLogLevel {
  NotSet = 0,
  // Trace doesn't exist in a Python world, so set it to "a bit lower than Debug".
  Trace = 5,
  Debug = 10,
  Info = 20,
  Warn = 30,
  Error = 40,
  Critical = 50,
}

impl From<log::Level> for PythonLogLevel {
  fn from(level: log::Level) -> Self {
    match level {
      log::Level::Error => PythonLogLevel::Error,
      log::Level::Warn => PythonLogLevel::Warn,
      log::Level::Info => PythonLogLevel::Info,
      log::Level::Debug => PythonLogLevel::Debug,
      log::Level::Trace => PythonLogLevel::Trace,
    }
  }
}

impl From<PythonLogLevel> for log::LevelFilter {
  fn from(level: PythonLogLevel) -> Self {
    match level {
      PythonLogLevel::NotSet => log::LevelFilter::Off,
      PythonLogLevel::Trace => log::LevelFilter::Trace,
      PythonLogLevel::Debug => log::LevelFilter::Debug,
      PythonLogLevel::Info => log::LevelFilter::Info,
      PythonLogLevel::Warn => log::LevelFilter::Warn,
      PythonLogLevel::Error => log::LevelFilter::Error,
      // Rust doesn't have a Critical, so treat them like Errors.
      PythonLogLevel::Critical => log::LevelFilter::Error,
    }
  }
}
