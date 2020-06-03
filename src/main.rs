#[macro_use]
extern crate lazy_static;
extern crate async_std;
extern crate async_trait;
extern crate clap;
extern crate colored;
extern crate ctrlc;
extern crate fslock;
extern crate juno;
extern crate rand;
extern crate semver;

mod models;
mod service;
mod utils;

use std::{env::current_dir, sync::Mutex};

use async_std::task;

use clap::{App, Arg};

use utils::{
	constants,
	logger::{self, LogLevel},
};

lazy_static! {
	static ref CONNECTION_TYPE: Mutex<u8> = Mutex::new(0);
}

#[allow(clippy::collapsible_if)]
#[async_std::main]
async fn main() {
	let args = App::new(constants::APP_NAME)
		.version(constants::APP_VERSION)
		.author(constants::APP_AUTHORS)
		.about("Micro-services framework")
		.arg(
			Arg::with_name("socket-location")
				.conflicts_with("port")
				.short("s")
				.long("socket-location")
				.takes_value(true)
				.value_name("FILE")
				.help("Sets the location of the socket to be created"),
		)
		.arg(
			Arg::with_name("port")
				.conflicts_with("socket-location")
				.short("p")
				.long("port")
				.takes_value(true)
				.value_name("PORT")
				.help("Sets the port for the socket to listen to"),
		)
		.arg(
			Arg::with_name("bind-addr")
				.conflicts_with("socket-location")
				.long("bind-addr")
				.takes_value(true)
				.value_name("BIND-ADDR")
				.help("Sets the binding address for the socket to listen to"),
		)
		.arg(
			Arg::with_name("V")
				.short("V")
				.multiple(true)
				.help("Sets the level of verbosity (max 3). Eg: -VVV for the highest logging level"),
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

	let mut port = String::from(args.value_of("bind-addr").unwrap_or("127.0.0.1"));
	port.push(':');
	port.push_str(args.value_of("port").unwrap_or("2203"));

	let socket_location = String::from(
		args.value_of("socket-location")
			.unwrap_or(default_socket_location),
	);

	let verbosity = match args.occurrences_of("V") {
		0 => LogLevel::Warn,
		1 => LogLevel::Debug,
		2 => LogLevel::Info,
		_ => LogLevel::Verbose,
	};
	logger::set_verbosity(verbosity);

	let mut connection_type = CONNECTION_TYPE.lock().unwrap();

	if cfg!(target_family = "windows") {
		if args.value_of("socket-location").is_some() {
			logger::error("Listening on unix sockets are not supported on windows");
			return;
		} else {
			*connection_type = constants::connection_types::INET_SOCKET;
		}
	} else {
		if args.value_of("port").is_some() {
			*connection_type = constants::connection_types::INET_SOCKET;
		} else {
			*connection_type = constants::connection_types::UNIX_SOCKET;
		}
	}

	if *connection_type == constants::connection_types::UNIX_SOCKET {
		logger::info(&format!(
			"Starting {} on socket location {}",
			constants::APP_NAME,
			socket_location,
		));

		ctrlc::set_handler(move || task::block_on(on_exit()))
			.expect("Unable to set Ctrl-C handler");
		drop(connection_type);
		if let Err(err) = service::start(&socket_location).await {
			logger::error(&format!("Error creating socket: {}", err));
		}
	} else if *connection_type == constants::connection_types::INET_SOCKET {
		logger::info(&format!(
			"Starting {} on port {}",
			constants::APP_NAME,
			port,
		));

		ctrlc::set_handler(move || task::block_on(on_exit()))
			.expect("Unable to set Ctrl-C handler");
		drop(connection_type);
		if let Err(err) = service::start(&port).await {
			logger::error(&format!("Error opening socket: {}", err));
		}
	}
}

fn get_connection_type() -> u8 {
	*CONNECTION_TYPE.lock().unwrap()
}

async fn on_exit() {
	logger::warn("Recieved exit code. Initiating shutdown process");
	service::on_exit().await;
}
