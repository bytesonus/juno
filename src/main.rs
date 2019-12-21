extern crate async_std;
extern crate clap;

use clap::{crate_authors, crate_name, crate_version, App, Arg};
use std::io::{BufReader, BufRead, Error};
use std::os::unix::net::{UnixListener, UnixStream};
use async_std::path::Path;
use async_std::task;
use async_std::fs::remove_file;

mod constants;

async fn handle_client(stream: UnixStream) {
	let stream = BufReader::new(stream);
	for line in stream.lines() {
		println!("{}", line.unwrap());
	}
}

fn main() {
	let args = App::new(crate_name!())
		.version(crate_version!())
		.author(crate_authors!())
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
		println!("{}", crate_version!());
		return;
	}

	let socket_location = args
		.value_of("socket-location")
		.unwrap_or(constants::DEFAULT_SOCKET_LOCATION);

	let socket_task = on_start(Path::new(socket_location));

	match task::block_on(socket_task) {
		Ok(_) => (),
		Err(err) => {
			println!("Error creating socket: {}", err);
			return;
		}
	}
}

async fn on_start(socket_path: &Path) -> Result<(), Error> {

	if socket_path.exists().await {
		remove_file(socket_path).await?;
	}

	// let socket_server = UnixListener::bind(socket_path).await?;
	// let mut incoming = socket_server.incoming();

	// while let Some(stream) = incoming.next().await {
	// 	let mut stream = stream?;
	// 	stream.write_all(b"hello world").await?;
	// }

	let socket_server = UnixListener::bind(socket_path)?;

	for stream in socket_server.incoming() {
		let stream = stream?;
		task::spawn(async {
			handle_client(stream).await;
		});
	}
	Ok(())
}
