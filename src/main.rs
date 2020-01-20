#[macro_use]
extern crate lazy_static;
extern crate async_std;
extern crate clap;
extern crate colored;
extern crate ctrlc;
extern crate file_lock;
extern crate rand;
extern crate semver;

mod models;
mod service;
mod utils;

use async_std::path::Path;
use async_std::task;

use clap::{App, Arg};

use service::socket_server;
use utils::constants;
use utils::logger::{self, LogLevel};

fn main() {
	let args = App::new(constants::APP_NAME)
		.version(constants::APP_VERSION)
		.author(constants::APP_AUTHORS)
		.about("Micro-services framework")
		.arg(
			Arg::with_name("socket-location")
				.short("s")
				.long("socket-location")
				.takes_value(true)
				.value_name("FILE")
				.default_value(constants::DEFAULT_SOCKET_LOCATION)
				.help("Sets the location of the socket to be created"),
		)
		.arg(
			Arg::with_name("V")
				.short("V")
				.multiple(true)
				.help("Sets the level of verbosity (max 3)"),
		)
		.arg(
			Arg::with_name("modules-location")
				.short("m")
				.long("modules-location")
				.takes_value(true)
				.value_name("DIR")
				.default_value(constants::DEFAULT_MODULES_LOCATION)
				.help("Sets the location of the modules to run"),
		)
		.arg(
			Arg::with_name("version")
				.short("v")
				.long("version")
				.help("Prints version information"),
		)
		.get_matches();

	if args.is_present("version") {
		println!("{}", constants::APP_VERSION);
		return;
	}

	let socket_location = args
		.value_of("socket-location")
		.unwrap_or(constants::DEFAULT_SOCKET_LOCATION);

	let verbosity = match args.occurrences_of("V") {
		0 => LogLevel::Warn,
		1 => LogLevel::Debug,
		2 => LogLevel::Info,
		3 | _ => LogLevel::Verbose,
	};
	logger::set_verbosity(verbosity);

	logger::info(&format!(
		"Starting {} with socket location {}",
		constants::APP_NAME,
		socket_location
	));

	// ctrlc::set_handler(move || task::block_on(on_exit())).expect("Unable to set Ctrl-C handler");

	let socket_task = socket_server::listen(Path::new(socket_location));

	if let Err(err) = task::block_on(socket_task) {
		logger::error(&format!("Error creating socket: {}", err));
	}
}

#[allow(dead_code)]
async fn on_exit() {
	();
}
