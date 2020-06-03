pub mod console_logger;

use console_logger::ConsoleLogger;
use std::sync::RwLock;

#[derive(Clone, Copy)]
pub enum LogLevel {
	Verbose = 1,
	Info = 2,
	Debug = 3,
	Warn = 4,
	Error = 5,
}

impl LogLevel {
	pub fn to_string(&self) -> &str {
		match &self {
			LogLevel::Verbose => "VERBOSE",
			LogLevel::Info => "INFO",
			LogLevel::Debug => "DEBUG",
			LogLevel::Warn => "WARN",
			LogLevel::Error => "ERROR",
		}
	}
}

trait Logger {
	fn verbose(&self, data: &str) {
		self.write(LogLevel::Verbose, data);
	}

	fn info(&self, data: &str) {
		self.write(LogLevel::Info, data);
	}

	fn debug(&self, data: &str) {
		self.write(LogLevel::Debug, data);
	}

	fn warn(&self, data: &str) {
		self.write(LogLevel::Warn, data);
	}

	fn error(&self, data: &str) {
		self.write(LogLevel::Error, data);
	}

	fn write(&self, log_level: LogLevel, data: &str);
}

lazy_static! {
	static ref DEFAULT_LOGGER: RwLock<ConsoleLogger> =
		RwLock::new(ConsoleLogger::new(LogLevel::Verbose));
}

pub fn verbose(data: &str) {
	DEFAULT_LOGGER.read().unwrap().verbose(data);
}

pub fn info(data: &str) {
	DEFAULT_LOGGER.read().unwrap().info(data);
}

pub fn debug(data: &str) {
	DEFAULT_LOGGER.read().unwrap().debug(data);
}

pub fn warn(data: &str) {
	DEFAULT_LOGGER.read().unwrap().warn(data);
}

pub fn error(data: &str) {
	DEFAULT_LOGGER.read().unwrap().error(data);
}

pub fn set_verbosity(log_level: LogLevel) {
	DEFAULT_LOGGER.write().unwrap().set_verbosity(log_level);
}
