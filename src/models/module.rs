use futures::channel::mpsc::{unbounded, UnboundedReceiver, UnboundedSender};
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
	socket_sender: UnboundedSender<String>,
	socket_receiver: UnboundedReceiver<String>,
}

#[allow(dead_code)]
impl Module {
	pub fn new(socket: UnixStream) -> Self {
		let (sender, receiver) = unbounded::<String>();
		Module {
			registered: false,
			module_id: String::new(),
			version: String::new(),
			dependencies: HashMap::new(),
			declared_functions: vec![],
			registered_hooks: vec![],
			socket,
			socket_sender: sender,
			socket_receiver: receiver,
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

	pub fn send(&mut self, data: String) {
		self.socket_sender.send(data);
	}

	pub async fn read_data_loop(&self) {
		let reader = BufReader::new(&self.socket);
		let mut lines = reader.lines();

		while let Some(line) = lines.next().await {
			if let Ok(line) = line {
				data_handler::handle_request(&self, &line);
			}
		}
	}

	pub async fn write_data_loop(&self) {
		let mut socket = &self.socket;
		let mut rec = &self.socket_receiver;

		while let Some(data) = rec.next().await {
			if let Err(err) = socket.write_all(data.as_bytes()).await {
				println!("Error while writing to socket: {}", err);
			}
		}
	}
}
