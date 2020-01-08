use crate::utils::constants;

use std::collections::HashMap;

use futures::channel::mpsc::UnboundedSender;
use futures_util::sink::SinkExt;

lazy_static! {
	pub static ref GOTHAM_MODULE: Module = Module::internal(
		0,
		String::from(constants::APP_NAME),
		String::from(constants::APP_VERSION),
	);
}

pub struct Module {
	registered: bool,
	module_uuid: u128,
	module_id: String,
	version: String,
	dependencies: HashMap<String, String>,
	declared_functions: Vec<String>,
	// These are the (global) hooks that this particular module is listening for
	registered_hooks: Vec<String>,

	module_sender: Option<UnboundedSender<String>>,
}

#[allow(dead_code)]
impl Module {
	pub fn new(
		module_uuid: u128,
		module_id: String,
		version: String,
		module_sender: UnboundedSender<String>,
	) -> Self {
		Module {
			registered: false,
			module_uuid,
			module_id,
			version,
			dependencies: HashMap::new(),
			declared_functions: vec![],
			registered_hooks: vec![],

			module_sender: Some(module_sender),
		}
	}

	fn internal(module_uuid: u128, module_id: String, version: String) -> Self {
		Module {
			registered: true,
			module_uuid,
			module_id,
			version,
			dependencies: HashMap::new(),
			declared_functions: vec![],
			registered_hooks: vec![],

			module_sender: None,
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

	// Exposing module_uuid
	pub fn get_module_uuid(&self) -> &u128 {
		&self.module_uuid
	}
	pub fn set_module_uuid(&mut self, module_uuid: u128) {
		self.module_uuid = module_uuid;
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
	pub fn get_dependencies(&self) -> &HashMap<String, String> {
		&self.dependencies
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

	pub async fn send(&self, data: String) {
		let sender = &self.module_sender;

		if let None = sender {
			return;
		}

		let mut sender = sender.as_ref().unwrap();
		let result = sender.send(data).await;
		if let Err(error) = result {
			println!("Error queing data to module: {}", error);
		}
	}

	pub async fn close_sender(&self) {
		let sender = &self.module_sender;

		if let None = sender {
			return;
		}

		let mut sender = sender.as_ref().unwrap();
		let result = sender.close().await;
		if let Err(error) = result {
			println!("Error closing module's sending queue: {}", error);
			return;
		}
	}
}
