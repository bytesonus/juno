use crate::utils::{constants, logger};

use async_std::{fs::remove_file, io::Result};

use futures::future;

use file_lock::FileLock;

pub mod data_handler;
pub mod module_runner;
pub mod socket_server;

pub async fn start(socket_path: &str, modules_path: &str) -> Result<()> {
	// Try to aquire a lock on the lock file first.
	let mut lock_file_path = socket_path.to_owned();
	lock_file_path.push_str(".lock");

	logger::verbose("Attempting to aquire lock file");
	let file_lock = match FileLock::lock(&lock_file_path, false, true) {
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
	logger::verbose("Lock file aquired.");

	let socket_listener_future = socket_server::listen(&socket_path);
	let module_runner_future = module_runner::listen(&modules_path);

	let (socket_listener_result, module_runner_result) =
		future::join(socket_listener_future, module_runner_future).await;
	logger::verbose(
		"Socket server and module runner have finished executing. Unlocking lock file...",
	);

	file_lock.unlock()?;
	logger::verbose("Lock file unlocked. Removing lock file...");

	remove_file(lock_file_path).await?;
	logger::verbose("Lock file removed. Removing socket file...");

	remove_file(socket_path).await?;
	logger::verbose("Socket file removed.");

	socket_listener_result?;
	module_runner_result?;

	Ok(())
}

pub async fn on_exit() {
	module_runner::on_exit().await;
	socket_server::on_exit().await;
}
