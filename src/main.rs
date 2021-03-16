mod models;
mod service;
mod utils;

use std::env::current_dir;

use clap::{App, Arg};

use log::LevelFilter;
use simple_logger::SimpleLogger;
use tokio::{signal, task};
use utils::{constants, ConnectionType};

#[tokio::main]
async fn main() {
	let args =
		App::new(constants::APP_NAME)
			.version(constants::APP_VERSION)
			.author(constants::APP_AUTHORS)
			.about(constants::APP_DESCRIPTION)
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
			.arg(Arg::with_name("V").short("V").multiple(true).help(
				"Sets the level of verbosity (max 3). Eg: -VVV for the highest logging level",
			))
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
	let default_socket_location =
		default_socket_location.as_os_str().to_str().unwrap();

	let mut port =
		String::from(args.value_of("bind-addr").unwrap_or("127.0.0.1"));
	port.push(':');
	port.push_str(args.value_of("port").unwrap_or("2203"));

	let socket_location = String::from(
		args.value_of("socket-location")
			.unwrap_or(default_socket_location),
	);

	SimpleLogger::new()
		.with_level(match args.occurrences_of("V") {
			0 => LevelFilter::Warn,
			1 => LevelFilter::Debug,
			2 => LevelFilter::Info,
			_ => LevelFilter::Trace,
		})
		.init()
		.unwrap();

	let connection_type;
	if cfg!(target_family = "windows") {
		if args.value_of("socket-location").is_some() {
			log::error!(
				"Listening on unix sockets are not supported on windows"
			);
			return;
		} else {
			connection_type = ConnectionType::UnixSocket;
		}
	} else {
		if args.value_of("port").is_some() {
			connection_type = ConnectionType::InetSocket;
		} else {
			connection_type = ConnectionType::UnixSocket;
		}
	}

	let exit_connection_type = connection_type.clone();
	task::spawn(async move {
		signal::ctrl_c()
			.await
			.expect("Unable to set Ctrl-C handler");
		on_exit(exit_connection_type).await;
	});

	if connection_type == ConnectionType::UnixSocket {
		log::info!(
			"Starting {} on socket location {}",
			constants::APP_NAME,
			socket_location,
		);
	} else if connection_type == ConnectionType::InetSocket {
		log::info!("Starting {} on port {}", constants::APP_NAME, port);
	}

	if let Err(err) = service::start(connection_type, &socket_location).await {
		log::error!("Error creating socket: {}", err);
	}
}

async fn on_exit(connection_type: ConnectionType) {
	log::warn!("Recieved exit code. Initiating shutdown process");
	service::on_exit(connection_type).await;
}
