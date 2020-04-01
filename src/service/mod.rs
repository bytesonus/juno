use crate::utils::{constants, logger};

use async_std::{fs::remove_file, io::Result, net::TcpListener};

use fslock::LockFile;

pub mod data_handler;
pub mod socket_server;

pub async fn start(socket_path: &str) -> Result<()> {
	// Make sure no other instances of the application is running
	if crate::get_connection_type() == constants::communication_types::UNIX_SOCKET {
		let mut lock_file_path = socket_path.to_string();
		lock_file_path.push_str(".lock");

		logger::verbose("Attempting to aquire lock file");
		let mut file_lock = LockFile::open(&lock_file_path)?;

		if !file_lock.try_lock()? {
			logger::error(&format!(
				"Unable to aquire socket file lock. Are there any other instances of {} running?",
				constants::APP_NAME
			));
			panic!("Exiting...");
		};
		logger::verbose("Lock file aquired.");

		let socket_listener_result = socket_server::listen(&socket_path).await;

		logger::verbose("Socket server has finished executing. Unlocking lock file...");

		file_lock.unlock()?;
		logger::verbose("Lock file unlocked. Removing lock file...");

		remove_file(lock_file_path).await?;
		logger::verbose("Lock file removed. Removing socket file...");

		remove_file(socket_path).await?;
		logger::verbose("Socket file removed.");

		socket_listener_result?;
	} else if crate::get_connection_type() == constants::communication_types::INET_SOCKET {
		let result = TcpListener::bind(socket_path).await;
		if result.is_err() {
			logger::error(&format!(
				"Unable to open port '{}'. Are there any other instances of {} running?",
				socket_path,
				constants::APP_NAME
			));
			panic!("Exiting...");
		}
		drop(result);
		logger::verbose("Port is available");

		socket_server::listen(&socket_path).await?;
	} else {
		panic!("Any other connection type other than INet sockets and Unix Sockets are not implemented yet");
	}

	Ok(())
}

pub async fn on_exit() {
	socket_server::on_exit().await;
}
