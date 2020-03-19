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

use std::env::current_dir;

use async_std::task;

use clap::{App, Arg};

use utils::{
	constants,
	logger::{self, LogLevel},
};

#[async_std::main]
async fn main() {
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

	let mut default_socket_location = current_dir().unwrap();
	default_socket_location.push(constants::DEFAULT_SOCKET_LOCATION);
	let default_socket_location = default_socket_location.as_os_str().to_str().unwrap();

	let mut default_modules_location = current_dir().unwrap();
	default_modules_location.push(constants::DEFAULT_MODULES_LOCATION);
	let default_modules_location = default_modules_location.as_os_str().to_str().unwrap();

	let socket_location = String::from(
		args.value_of("socket-location")
			.unwrap_or(default_socket_location),
	);

	let modules_location = String::from(
		args.value_of("modules-location")
			.unwrap_or(default_modules_location),
	);

	let verbosity = match args.occurrences_of("V") {
		0 => LogLevel::Warn,
		1 => LogLevel::Debug,
		2 => LogLevel::Info,
		_ => LogLevel::Verbose,
	};
	logger::set_verbosity(verbosity);

	logger::info(&format!(
		"Starting {} with socket location {} and modules location {}",
		constants::APP_NAME,
		socket_location,
		modules_location,
	));

	ctrlc::set_handler(move || task::block_on(on_exit())).expect("Unable to set Ctrl-C handler");

	if let Err(err) = service::start(&socket_location, &modules_location).await {
		logger::error(&format!("Error creating socket: {}", err));
	}
}

async fn on_exit() {
	logger::warn("Recieved exit code. Initiating shutdown process");
	service::on_exit().await;
}
