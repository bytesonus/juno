use crate::{service::data_handler, utils::logger};

use futures::channel::mpsc::{UnboundedReceiver, UnboundedSender};
use futures_util::sink::SinkExt;

use async_std::{io::BufReader, os::unix::net::UnixStream, prelude::*};

#[allow(dead_code)]
pub struct ModuleComm {
	module_uuid: u128,

	socket: UnixStream,
	socket_sender: UnboundedSender<String>,
}

#[allow(dead_code)]
impl ModuleComm {
	pub fn new(
		module_uuid: u128,
		socket: UnixStream,
		socket_sender: UnboundedSender<String>,
	) -> Self {
		ModuleComm {
			module_uuid,
			socket,
			socket_sender,
		}
	}

	pub fn get_uuid(&self) -> &u128 {
		&self.module_uuid
	}

	pub fn clone_sender(&self) -> UnboundedSender<String> {
		self.socket_sender.clone()
	}

	pub async fn send(&self, data: String) {
		let mut sender = &self.socket_sender;
		let result = sender.send(data).await;
		if let Err(error) = result {
			println!("Error queing data to module: {}", error);
		}
	}

	pub async fn close_sender(&self) {
		let mut sender = &self.socket_sender;
		let result = sender.close().await;
		if let Err(error) = result {
			println!("Error closing module's sending queue: {}", error);
			return;
		}
	}

	pub async fn read_data_loop(&self) {
		let reader = BufReader::new(&self.socket);
		let mut lines = reader.lines();

		while let Some(Ok(line)) = lines.next().await {
			data_handler::handle_request(&self, line).await;
		}

		logger::verbose("Closed socket. Disconnecting module...");
		data_handler::on_module_disconnected(&self).await;
		logger::verbose("Module disconnected. Closing sender...");
		self.close_sender().await;
		logger::verbose("Sender closed");
	}

	pub async fn write_data_loop(&self, receiver: &mut UnboundedReceiver<String>) {
		let mut socket = &self.socket;

		while let Some(data) = receiver.next().await {
			if let Err(err) = socket.write_all(data.as_bytes()).await {
				println!("Error while writing to socket: {}", err);
			}
		}
	}
}
