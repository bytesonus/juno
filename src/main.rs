extern crate async_std;
extern crate clap;
#[macro_use]
extern crate lazy_static;
extern crate rand;

mod models;
mod service;
mod utils;

use async_std::path::Path;
use async_std::task;
use clap::{App, Arg};
use service::socket_server;
use utils::constants;

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
				.help("Sets the level of verbosity"),
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

	let socket_task = socket_server::listen(Path::new(socket_location));

	if let Err(err) = task::block_on(socket_task) {
		println!("Error creating socket: {}", err);
	}
}
