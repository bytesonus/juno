use super::data_handler;
use crate::models::ModuleComm;

use async_std::fs::remove_file;
use async_std::io::Result;
use async_std::os::unix::net::{UnixListener, UnixStream};
use async_std::path::Path;
use async_std::prelude::StreamExt;
use async_std::task;

use futures::channel::mpsc::unbounded;
use futures::future;

use rand::{thread_rng, Rng};

pub async fn listen(socket_path: &Path) -> Result<()> {
	// TODO Try to aquire a lock on the lock file first.
	/*
	let mut lock_file_path = socket_path.to_str().unwrap().to_owned();
	lock_file_path.push_str(".lock");
	
	let _file_lock = match FileLock::lock(&lock_file_path, false, true) {
		Ok(lock) => lock,
		// If lock fails, return an error
		Err(err) => panic!("Error getting file lock: {}", err)
	};
	*/

	// File lock is aquired. If the unix socket exists, then it's clearly a dangling socket. Feel free to delete it
	if socket_path.exists().await {
		remove_file(socket_path).await?;
	}

	let socket_server = UnixListener::bind(socket_path).await?;
	let mut incoming = socket_server.incoming();

	while let Some(stream) = incoming.next().await {
		task::spawn(async {
			handle_client(stream).await;
		});
	}

	Ok(())
}

async fn handle_client(stream: Result<UnixStream>) {
	if let Err(_) = stream {
		println!("Error occured while opening socket");
		return;
	}

	let stream = stream.unwrap();
	let (sender, mut receiver) = unbounded::<String>();

	let mut uuid: u128 = thread_rng().gen();
	while uuid != 0 && data_handler::is_uuid_exists(&uuid).await {
		uuid = thread_rng().gen();
	}
	let module_comm = ModuleComm::new(uuid, stream, sender);

	let read_future = module_comm.read_data_loop();
	let write_future = module_comm.write_data_loop(&mut receiver);

	future::join(read_future, write_future).await;
}
