use super::data_handler;
use crate::{
	models::ModuleComm,
	utils::{constants, logger},
};

use async_std::{
	fs::remove_file,
	io::Result,
	os::unix::net::{UnixListener, UnixStream},
	path::Path,
	prelude::StreamExt,
	task,
};

use futures::{
	channel::mpsc::{unbounded, UnboundedReceiver, UnboundedSender},
	future,
};

use rand::{thread_rng, Rng};

use file_lock::FileLock;

lazy_static! {
	static ref CLOSE_HANDLERS: (UnboundedSender<bool>, UnboundedReceiver<bool>) =
		unbounded::<bool>();
}

pub async fn listen(socket_path: &Path) -> Result<()> {
	// Try to aquire a lock on the lock file first.
	let mut lock_file_path = socket_path.to_str().unwrap().to_owned();
	lock_file_path.push_str(".lock");

	logger::verbose("Attempting to aquire lock file");
	let _file_lock = match FileLock::lock(&lock_file_path, false, true) {
		Ok(lock) => lock,
		// If lock fails, return an error
		Err(_) => {
			logger::error(&format!(
				"Unable to aquire socket file lock. Are there any other instances of {} running?",
				constants::APP_NAME
			));
			panic!("Exiting...");
		}
	};
	logger::verbose("Lock file aquired");

	// File lock is aquired. If the unix socket exists, then it's clearly a dangling socket. Feel free to delete it
	if socket_path.exists().await {
		logger::verbose("Removing existing unix socket");
		remove_file(socket_path).await?;
	}

	let socket_server = UnixListener::bind(socket_path).await?;
	let mut incoming = socket_server.incoming();

	logger::verbose("Listening for socket connections...");
	while let Some(stream) = incoming.next().await {
		logger::info("Socket connected");
		task::spawn(async {
			handle_client(stream).await;
		});
	}

	Ok(())
}

async fn handle_client(stream: Result<UnixStream>) {
	if let Err(_) = stream {
		logger::error("Error occured while opening socket");
		return;
	}

	let stream = stream.unwrap();
	let (sender, mut receiver) = unbounded::<String>();
	logger::verbose("New MPSC channel created");

	let mut uuid: u128 = thread_rng().gen();
	while uuid != 0 && data_handler::is_uuid_exists(&uuid).await {
		uuid = thread_rng().gen();
	}
	logger::info(&format!("New connection assigned ID {}", uuid));
	let module_comm = ModuleComm::new(uuid, stream, sender);

	logger::verbose(&format!("Polling connection ID {}", uuid));
	let read_future = module_comm.read_data_loop();
	let write_future = module_comm.write_data_loop(&mut receiver);

	future::join(read_future, write_future).await;
	logger::info(&format!("Connection with ID {} disconnected", uuid));
}
