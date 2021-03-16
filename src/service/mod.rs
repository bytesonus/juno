use crate::utils::{constants, ConnectionType};

use fslock::LockFile;
use tokio::{fs::remove_file, io::Result, net::TcpListener};

pub mod data_handler;
pub mod socket_server;

pub async fn start(
	connection_type: ConnectionType,
	socket_path: &str,
) -> Result<()> {
	// Make sure no other instances of the application is running
	if connection_type == ConnectionType::UnixSocket {
		let mut lock_file_path = socket_path.to_string();
		lock_file_path.push_str(".lock");

		log::trace!("Attempting to aquire lock file");
		let mut file_lock = LockFile::open(&lock_file_path)?;

		if !file_lock.try_lock()? {
			log::error!(
				"Unable to aquire socket file lock. Are there any other instances of {} running?",
				constants::APP_NAME
			);
			panic!("Exiting...");
		};
		log::trace!("Lock file aquired.");

		let socket_listener_result =
			socket_server::listen(connection_type, &socket_path).await;

		log::trace!(
			"Socket server has finished executing. Unlocking lock file..."
		);

		file_lock.unlock()?;
		log::trace!("Lock file unlocked. Removing lock file...");

		remove_file(lock_file_path).await?;
		log::trace!("Lock file removed. Removing socket file...");

		remove_file(socket_path).await?;
		log::trace!("Socket file removed.");

		socket_listener_result?;
	} else if connection_type == ConnectionType::InetSocket {
		let result = TcpListener::bind(socket_path).await;
		if result.is_err() {
			log::error!(
				"Unable to open port '{}'. Are there any other instances of {} running?",
				socket_path,
				constants::APP_NAME
			);
			panic!("Exiting...");
		}
		drop(result);
		log::trace!("Port is available");

		socket_server::listen(connection_type, &socket_path).await?;
	}

	Ok(())
}

pub async fn on_exit(connection_type: ConnectionType) {
	socket_server::on_exit(connection_type).await;
}
