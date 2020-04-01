use crate::utils::{constants, logger};

use std::collections::HashMap;

use futures::channel::mpsc::UnboundedSender;
use futures_util::sink::SinkExt;

use semver::{Version, VersionReq};

lazy_static! {
	pub static ref GOTHAM_MODULE: Module = Module::internal(
		0,
		String::from(constants::APP_NAME),
		Version::parse(constants::APP_VERSION).unwrap(),
	);
}

pub struct Module {
	registered: bool,
	module_uuid: u128,
	module_id: String,
	version: Version,
	dependencies: HashMap<String, VersionReq>,
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
		version: Version,
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

	fn internal(module_uuid: u128, module_id: String, version: Version) -> Self {
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
	pub fn get_version(&self) -> &Version {
		&self.version
	}
	pub fn set_version(&mut self, version: Version) {
		self.version = version;
	}

	// Exposing dependencies
	pub fn set_dependencies(&mut self, dependencies: HashMap<String, VersionReq>) {
		self.dependencies = dependencies;
	}
	pub fn get_dependencies(&self) -> &HashMap<String, VersionReq> {
		&self.dependencies
	}
	pub fn get_dependency(&mut self, module_id: &str) -> Option<&VersionReq> {
		self.dependencies.get(&module_id.to_string())
	}

	// Exposing declared_functions
	pub fn declare_function(&mut self, function_name: String) {
		self.declared_functions.push(function_name);
	}
	pub fn is_function_declared(&self, function_name: &str) -> bool {
		self.declared_functions.contains(&function_name.to_string())
	}

	// Exposing registered_hooks
	pub fn register_hook(&mut self, hook_name: String) {
		self.registered_hooks.push(hook_name);
	}
	pub fn is_hook_registered(&self, hook_name: &str) -> bool {
		self.registered_hooks.contains(&hook_name.to_string())
	}

	pub async fn send(&self, data: String) {
		let sender = &self.module_sender;

		if sender.is_none() {
			return;
		}

		let mut sender = sender.as_ref().unwrap();
		let result = sender.send(data).await;
		if let Err(error) = result {
			logger::error(&format!("Error queing data to module: {}", error));
		}
	}

	pub async fn close_sender(&self) {
		let sender = &self.module_sender;

		if sender.is_none() {
			return;
		}

		let mut sender = sender.as_ref().unwrap();
		let result = sender.close().await;
		if let Err(error) = result {
			logger::error(&format!("Error closing module's sending queue: {}", error));
			return;
		}
	}
}
