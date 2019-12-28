use futures::channel::mpsc::{UnboundedReceiver, UnboundedSender};
use futures_util::sink::SinkExt;

use async_std::io::BufReader;
use async_std::os::unix::net::UnixStream;
use async_std::prelude::StreamExt;
use async_std::prelude::*;
use std::collections::HashMap;

use crate::service::data_handler;

#[allow(dead_code)]
pub struct Module {
	registered: bool,
	module_id: String,
	version: String,
	dependencies: HashMap<String, String>,
	declared_functions: Vec<String>,
	// These are the (global) hooks that this particular module is listening for
	registered_hooks: Vec<String>,

	socket: UnixStream,
	socket_sender: Option<UnboundedSender<String>>,
}

#[allow(dead_code)]
impl Module {
	pub fn new(socket: UnixStream) -> Self {
		Module {
			registered: false,
			module_id: String::new(),
			version: String::new(),
			dependencies: HashMap::new(),
			declared_functions: vec![],
			registered_hooks: vec![],
			socket,
			socket_sender: None,
		}
	}

	// Exposing registered
	pub fn is_registered(&self) -> bool {
		self.registered
	}
	pub fn set_registered(&mut self, registered: bool) {
		self.registered = registered;
	}

	// Exposing module_id
	pub fn get_module_id(&self) -> &String {
		&self.module_id
	}
	pub fn set_module_id(&mut self, module_id: String) {
		self.module_id = module_id;
	}

	// Exposing version
	pub fn get_version(&self) -> &String {
		&self.version
	}
	pub fn set_version(&mut self, version: String) {
		self.version = version;
	}

	// Exposing dependencies
	pub fn set_dependencies(&mut self, dependencies: HashMap<String, String>) {
		self.dependencies = dependencies;
	}
	pub fn get_dependency(&mut self, module_id: &String) -> Option<&String> {
		self.dependencies.get(module_id)
	}

	// Exposing declared_functions
	pub fn declare_function(&mut self, function_name: String) {
		self.declared_functions.push(function_name);
	}
	pub fn is_function_declared(&self, function_name: &String) -> bool {
		self.declared_functions.contains(function_name)
	}

	// Exposing registered_hooks
	pub fn register_hook(&mut self, hook_name: String) {
		self.registered_hooks.push(hook_name);
	}
	pub fn is_hook_registered(&self, hook_name: &String) -> bool {
		self.registered_hooks.contains(hook_name)
	}

	pub fn set_sender(&mut self, sender: UnboundedSender<String>) {
		self.socket_sender = Some(sender);
	}

	pub async fn send(&self, data: String) {
		if let Some(sender) = &self.socket_sender {
			let mut sender = sender;
			let result = sender.send(data).await;
			if let Err(error) = result {
				println!("Error queing data to module: {}", error);
			}
		}
	}

	pub async fn close_sender(&self) {
		if let Some(sender) = &self.socket_sender {
			let mut sender = sender;
			let result = sender.close().await;
			if let Err(error) = result {
				println!("Error closing module's sending queue: {}", error);
				return;
			}
		}
	}

	pub async fn read_data_loop(&self) {
		let reader = BufReader::new(&self.socket);
		let mut lines = reader.lines();

		while let Some(line) = lines.next().await {
			if let Ok(line) = line {
				data_handler::handle_request(&self, line).await;
			}
		}

		self.close_sender().await;
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
