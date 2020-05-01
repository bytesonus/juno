use crate::{service::data_handler, utils::logger};
use juno::connection::Buffer;

use futures::channel::mpsc::{UnboundedReceiver, UnboundedSender};
use futures_util::sink::SinkExt;

#[cfg(target_family = "unix")]
use async_std::os::unix::net::UnixStream;
use async_std::{io::BufReader, net::TcpStream, prelude::*};

pub enum ModuleComm {
	#[cfg(target_family = "unix")]
	UnixSocketComms {
		module_uuid: u128,

		socket: UnixStream,
		socket_sender: UnboundedSender<String>,
	},
	InetSocketComms {
		module_uuid: u128,

		socket: TcpStream,
		socket_sender: UnboundedSender<String>,
	},
	InternalComms {
		module_uuid: u128,
		juno_sender: UnboundedSender<Buffer>,
		socket_sender: UnboundedSender<String>,
	},
}

impl ModuleComm {
	#[cfg(target_family = "unix")]
	pub fn new_unix_comm(
		module_uuid: u128,
		socket: UnixStream,
		socket_sender: UnboundedSender<String>,
	) -> Self {
		ModuleComm::UnixSocketComms {
			module_uuid,
			socket,
			socket_sender,
		}
	}
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
	pub fn new_internal_comm(
		module_uuid: u128,
		juno_sender: UnboundedSender<Buffer>,
		socket_sender: UnboundedSender<String>,
	) -> Self {
		ModuleComm::InternalComms {
			module_uuid,
			juno_sender,
			socket_sender,
		}
	}

	pub fn get_uuid(&self) -> &u128 {
		match self {
			#[cfg(target_family = "unix")]
			ModuleComm::UnixSocketComms { module_uuid, .. } => module_uuid,
			ModuleComm::InetSocketComms { module_uuid, .. } => module_uuid,
			ModuleComm::InternalComms { module_uuid, .. } => module_uuid,
		}
	}

	pub fn clone_sender(&self) -> UnboundedSender<String> {
		match self {
			#[cfg(target_family = "unix")]
			ModuleComm::UnixSocketComms { socket_sender, .. } => socket_sender.clone(),
			ModuleComm::InetSocketComms { socket_sender, .. } => socket_sender.clone(),
			ModuleComm::InternalComms { socket_sender, .. } => socket_sender.clone(),
		}
	}

	pub async fn send(&self, data: String) {
		let mut sender = match self {
			#[cfg(target_family = "unix")]
			ModuleComm::UnixSocketComms { socket_sender, .. } => socket_sender,
			ModuleComm::InetSocketComms { socket_sender, .. } => socket_sender,
			ModuleComm::InternalComms { socket_sender, .. } => socket_sender,
		};
		let result = sender.send(data).await;
		if let Err(error) = result {
			logger::error(&format!("Error queing data to module: {}", error));
		}
	}

	pub async fn close_sender(&self) {
		let mut sender = match self {
			#[cfg(target_family = "unix")]
			ModuleComm::UnixSocketComms { socket_sender, .. } => socket_sender,
			ModuleComm::InetSocketComms { socket_sender, .. } => socket_sender,
			ModuleComm::InternalComms { socket_sender, .. } => socket_sender,
		};
		let result = sender.close().await;
		if let Err(error) = result {
			logger::error(&format!("Error closing module's sending queue: {}", error));
			return;
		}
	}

	pub async fn read_data_loop(&self) {
		match self {
			#[cfg(target_family = "unix")]
			ModuleComm::UnixSocketComms { socket, .. } => {
				let reader = BufReader::new(socket);
				let mut lines = reader.lines();

				while let Some(Ok(line)) = lines.next().await {
					data_handler::handle_request(&self, line).await;
				}
			}
			ModuleComm::InetSocketComms { socket, .. } => {
				let reader = BufReader::new(socket);
				let mut lines = reader.lines();

				while let Some(Ok(line)) = lines.next().await {
					data_handler::handle_request(&self, line).await;
				}
			}
			_ => panic!("Cannot execute read-data loop on Internal comms"),
		};

		logger::verbose("Closed socket. Disconnecting module...");
		data_handler::on_module_disconnected(&self).await;
		logger::verbose("Module disconnected. Closing sender...");
		self.close_sender().await;
		logger::verbose("Sender closed");
	}

	pub(crate) async fn internal_read_loop(&self, mut receiver: UnboundedReceiver<Buffer>) {
		if let ModuleComm::InternalComms { .. } = self {
			while let Some(line) = receiver.next().await {
				let line = String::from_utf8(line).unwrap();
				data_handler::handle_request(&self, line).await;
			}

			logger::verbose("Internal comm receiver dropper. Disconnecting module...");
			data_handler::on_module_disconnected(&self).await;
			logger::verbose("Module disconnected. Closing sender...");
			self.close_sender().await;
			logger::verbose("Sender closed");
		} else {
			panic!("Cannot execute internal read-data loop on socket module comm");
		}
	}

	pub async fn write_data_loop(&self, receiver: &mut UnboundedReceiver<String>) {
		match self {
			#[cfg(target_family = "unix")]
			ModuleComm::UnixSocketComms { socket, .. } => {
				let mut socket = socket;

				while let Some(data) = receiver.next().await {
					if let Err(err) = socket.write_all(data.as_bytes()).await {
						logger::error(&format!("Error while writing to socket: {}", err));
					}
				}
			}
			ModuleComm::InetSocketComms { socket, .. } => {
				let mut socket = socket;

				while let Some(data) = receiver.next().await {
					if let Err(err) = socket.write_all(data.as_bytes()).await {
						logger::error(&format!("Error while writing to socket: {}", err));
					}
				}
			}
			ModuleComm::InternalComms { juno_sender, .. } => {
				let mut sender = juno_sender;

				while let Some(data) = receiver.next().await {
					if let Err(err) = sender.send(data.as_bytes().to_vec()).await {
						logger::error(&format!("Error while writing to socket: {}", err));
					}
				}
			}
		};
	}
}
