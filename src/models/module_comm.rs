use crate::service::data_handler;

use futures::{
	channel::mpsc::{UnboundedSender},
};
use futures_util::sink::SinkExt;
use tokio::io::{
	AsyncBufReadExt, AsyncRead, BufReader,
};

pub struct ModuleComm {
	module_uuid: u128,
	write_buffer_sender: UnboundedSender<String>,
}

impl ModuleComm {
	pub fn new(
		module_uuid: u128,
		write_buffer_sender: UnboundedSender<String>,
	) -> Self {
		Self {
			module_uuid,
			write_buffer_sender,
		}
	}

	pub fn get_uuid(&self) -> &u128 {
		&self.module_uuid
	}

	pub fn clone_sender(&self) -> UnboundedSender<String> {
		self.write_buffer_sender.clone()
	}

	pub async fn send(&self, data: String) {
		let mut sender = &self.write_buffer_sender;
		let result = sender.send(data).await;
		if let Err(error) = result {
			log::error!("Error queing data to module: {}", error);
		}
	}

	pub async fn close_sender(&self) {
		let mut sender = &self.write_buffer_sender;
		let result = sender.close().await;
		if let Err(error) = result {
			log::error!("Error closing module's sending queue: {}", error);
			return;
		}
	}

	pub async fn read_data_loop<TReader: AsyncRead + Unpin>(
		&self,
		module_input_reader: TReader,
	) {
		let reader = BufReader::new(module_input_reader);
		let mut lines = reader.lines();

		while let Ok(Some(line)) = lines.next_line().await {
			data_handler::handle_request(&self, line).await;
		}

		log::trace!("Closed socket. Disconnecting module...");
		data_handler::on_module_disconnected(&self).await;
		log::trace!("Module disconnected. Closing sender...");
		self.close_sender().await;
		log::trace!("Sender closed");
	}
}
