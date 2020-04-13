use crate::{service::data_handler, utils::logger};

use futures::channel::mpsc::{UnboundedReceiver, UnboundedSender};
use futures_util::sink::SinkExt;

use async_std::{io::BufReader, net::TcpStream, prelude::*};

pub enum ModuleComm {
	InetSocketComms {
		module_uuid: u128,

		socket: TcpStream,
		socket_sender: UnboundedSender<String>,
	},
}

impl ModuleComm {
	pub fn new_inet_comm(
		module_uuid: u128,
		socket: TcpStream,
		socket_sender: UnboundedSender<String>,
	) -> Self {
		ModuleComm::InetSocketComms {
			module_uuid,
			socket,
			socket_sender,
		}
	}

	pub fn get_uuid(&self) -> &u128 {
		match self {
			ModuleComm::InetSocketComms { module_uuid, .. } => module_uuid,
		}
	}

	pub fn clone_sender(&self) -> UnboundedSender<String> {
		match self {
			ModuleComm::InetSocketComms { socket_sender, .. } => socket_sender.clone(),
		}
	}

	pub async fn send(&self, data: String) {
		let mut sender = match self {
			ModuleComm::InetSocketComms { socket_sender, .. } => socket_sender.clone(),
		};
		let result = sender.send(data).await;
		if let Err(error) = result {
			logger::error(&format!("Error queing data to module: {}", error));
		}
	}

	pub async fn close_sender(&self) {
		let mut sender = match self {
			ModuleComm::InetSocketComms { socket_sender, .. } => socket_sender,
		};
		let result = sender.close().await;
		if let Err(error) = result {
			logger::error(&format!("Error closing module's sending queue: {}", error));
			return;
		}
	}

	pub async fn read_data_loop(&self) {
		match self {
			ModuleComm::InetSocketComms { socket, .. } => {
				let reader = BufReader::new(socket);
				let mut lines = reader.lines();

				while let Some(Ok(line)) = lines.next().await {
					data_handler::handle_request(&self, line).await;
				}
			}
		};

		logger::verbose("Closed socket. Disconnecting module...");
		data_handler::on_module_disconnected(&self).await;
		logger::verbose("Module disconnected. Closing sender...");
		self.close_sender().await;
		logger::verbose("Sender closed");
	}

	pub async fn write_data_loop(&self, receiver: &mut UnboundedReceiver<String>) {
		match self {
			ModuleComm::InetSocketComms { socket, .. } => {
				let mut socket = socket;

				while let Some(data) = receiver.next().await {
					if let Err(err) = socket.write_all(data.as_bytes()).await {
						logger::error(&format!("Error while writing to socket: {}", err));
					}
				}
			}
		};
	}
}
