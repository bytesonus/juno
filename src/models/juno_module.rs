use crate::{constants, models::Module, service::data_handler};
use juno::models::Value;
use juno::JunoModuleImpl;

use std::{collections::HashMap, sync::Arc};

use async_std::task;
use async_trait::async_trait;
use futures::{
	channel::mpsc::{UnboundedReceiver, UnboundedSender},
	StreamExt,
};
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
	on_data_handler: Option<Arc<JunoModuleImpl>>,
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
			on_data_handler: None,
		}
	}
}

#[async_trait]
impl BaseConnection for DirectConnection {
	async fn setup_connection(&mut self) -> Result<(), Error> {
		if self.connection_setup || self.read_data_receiver.is_none() {
			return Err(Error::Internal(String::from(
				"Cannot call setup_connection() more than once!",
			)));
		}

		if self.on_data_handler.is_none() {
			return Err(Error::Internal(String::from(
				"On data handler cannot be empty!",
			)));
		}

		let mut read_data_receiver = self.read_data_receiver.take().unwrap();
		let on_data_handler = self.on_data_handler.as_ref().unwrap().clone();

		task::spawn(async move {
			while let Some(data) = read_data_receiver.next().await {
				let juno_impl = on_data_handler.clone();
				task::spawn(async move {
					juno_impl.on_data(data).await;
				});
			}
		});

		self.connection_setup = true;
		Ok(())
	}

	async fn close_connection(&mut self) -> Result<(), Error> {
		if !self.connection_setup {
			panic!("Cannot close a connection that hasn't been established yet. Did you forget to call setup_connection()?");
		}
		Ok(())
	}

	async fn send(&mut self, buffer: Buffer) -> Result<(), Error> {
		if !self.connection_setup {
			panic!("Cannot send data to a connection that hasn't been established yet. Did you forget to await the call to setup_connection()?");
		}
		if buffer.is_empty() {
			return Ok(());
		}
		self.write_data_sender.send(buffer).await.unwrap();
		Ok(())
	}

	fn set_data_listener(&mut self, listener: Arc<JunoModuleImpl>) {
		self.on_data_handler = Some(listener);
	}

	fn get_data_listener(&self) -> &Option<Arc<JunoModuleImpl>> {
		&self.on_data_handler
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
	Value::Array(modules.into_iter().map(get_object_from_module).collect())
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
