use crate::{constants, models::Module, service::data_handler};
use juno::models::Value;

use std::collections::HashMap;

use async_std::task;
use async_trait::async_trait;
use futures::channel::mpsc::{UnboundedReceiver, UnboundedSender};
use futures_util::sink::SinkExt;
use juno::{
	connection::{BaseConnection, Buffer},
	protocol::BaseProtocol,
	Error, JunoModule,
};

pub(crate) struct DirectConnection {
	connection_setup: bool,
	read_data_receiver: Option<UnboundedReceiver<Buffer>>,
	write_data_sender: UnboundedSender<Buffer>,
}

impl DirectConnection {
	pub fn new(
		read_data_receiver: UnboundedReceiver<Buffer>,
		write_data_sender: UnboundedSender<Buffer>,
	) -> Self {
		DirectConnection {
			connection_setup: false,
			read_data_receiver: Some(read_data_receiver),
			write_data_sender,
		}
	}
}

#[async_trait]
impl BaseConnection for DirectConnection {
	async fn setup_connection(&mut self) -> Result<(), Error> {
		if self.connection_setup {
			panic!("Cannot call setup_connection() more than once!");
		}

		self.connection_setup = true;
		Ok(())
	}

	async fn close_connection(&mut self) {
		if !self.connection_setup {
			panic!("Cannot close a connection that hasn't been established yet. Did you forget to call setup_connection()?");
		}
	}

	async fn send(&mut self, buffer: Buffer) {
		if !self.connection_setup {
			panic!("Cannot send data to a connection that hasn't been established yet. Did you forget to await the call to setup_connection()?");
		}
		let mut sender = &self.write_data_sender.clone();
		if let Err(err) = sender.send(buffer).await {
			println!("Error attempting to send data to connection: {}", err);
		}
	}

	fn get_data_receiver(&mut self) -> UnboundedReceiver<Buffer> {
		if !self.connection_setup {
			panic!("Cannot get read sender to a connection that hasn't been established yet. Did you forget to await the call to setup_connection()?");
		}
		self.read_data_receiver.take().unwrap()
	}

	fn clone_write_sender(&self) -> UnboundedSender<Buffer> {
		if !self.connection_setup {
			panic!("Cannot get write sender of a connection that hasn't been established yet. Did you forget to await the call to setup_connection()?");
		}
		self.write_data_sender.clone()
	}
}

pub(crate) async fn setup_juno_module(
	read_data_receiver: UnboundedReceiver<Buffer>,
	write_data_sender: UnboundedSender<Buffer>,
) -> JunoModule {
	let mut module = JunoModule::new(
		BaseProtocol::default(),
		Box::new(DirectConnection::new(read_data_receiver, write_data_sender)),
	);

	module
		.initialize(constants::APP_NAME, constants::APP_VERSION, HashMap::new())
		.await
		.unwrap();

	module
		.declare_function("listModules", list_modules)
		.await
		.unwrap();

	module
		.declare_function("getModuleInfo", get_module_info)
		.await
		.unwrap();

	module
}

fn list_modules(_: HashMap<String, Value>) -> Value {
	let mut modules = task::block_on(data_handler::get_registered_modules());
	modules.extend(task::block_on(data_handler::get_unregistered_modules()));
	Value::Array(
		modules
			.into_iter()
			.map(get_object_from_module)
			.collect(),
	)
}

fn get_module_info(args: HashMap<String, Value>) -> Value {
	let module_id = args.get("moduleId");
	if module_id.is_none() {
		return Value::Null;
	}
	let module_id = module_id.unwrap().as_string();
	if module_id.is_none() {
		return Value::Null;
	}
	let module_id = module_id.unwrap();

	if let Some(module) = task::block_on(data_handler::get_module_by_id(module_id)) {
		get_object_from_module(module)
	} else {
		Value::Null
	}
}

fn get_object_from_module(module: Module) -> Value {
	let Module {
		module_id,
		registered,
		version,
		dependencies,
		declared_functions,
		registered_hooks,
		..
	} = module;
	Value::Object({
		let mut map = HashMap::new();

		map.insert(String::from("moduleId"), Value::String(module_id));
		map.insert(String::from("version"), Value::String(version.to_string()));
		map.insert(
			String::from("dependencies"),
			Value::Object(
				dependencies
					.into_iter()
					.map(|(key, value)| (key, Value::String(value.to_string())))
					.collect(),
			),
		);
		map.insert(String::from("registered"), Value::Bool(registered));
		map.insert(
			String::from("declaredFunctions"),
			Value::Array(declared_functions.into_iter().map(Value::String).collect()),
		);
		map.insert(
			String::from("registeredHooks"),
			Value::Array(registered_hooks.into_iter().map(Value::String).collect()),
		);

		map
	})
}
