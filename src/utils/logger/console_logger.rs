use colored::*;

use super::{LogLevel, Logger};

pub struct ConsoleLogger {
	verbosity: LogLevel,
}

impl ConsoleLogger {
	pub fn new(verbosity: LogLevel) -> Self {
		ConsoleLogger { verbosity }
	}

	pub fn set_verbosity(&mut self, verbosity: LogLevel) {
		self.verbosity = verbosity;
	}
}

impl Logger for ConsoleLogger {
	fn write(&self, log_level: LogLevel, data: &str) {
		let myself = self.verbosity as u8;
		let other = log_level as u8;
		if myself <= other {
			let log_level = match log_level {
				LogLevel::Verbose => log_level.to_string().green(),
				LogLevel::Info => log_level.to_string().blue(),
				LogLevel::Debug => log_level.to_string().yellow(),
				LogLevel::Warn => log_level.to_string().on_yellow().black(),
				LogLevel::Error => log_level.to_string().on_red().white(),
			};
			println!("[{}]: {}", log_level, data);
		}
	}
}
