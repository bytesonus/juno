use crate::models::module;
use crate::models::Module;
use crate::models::ModuleComm;
use crate::utils::constants::errors;
use crate::utils::constants::gotham_hooks;
use crate::utils::constants::request_keys;
use crate::utils::constants::request_types;

use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use async_std::sync::Mutex;

use serde_json::{json, Map, Value};

lazy_static! {
	static ref REGISTERED_MODULES: Mutex<HashMap<String, Module>> = Mutex::new(HashMap::new());
	static ref UNREGISTERED_MODULES: Mutex<HashMap<String, Module>> = Mutex::new(HashMap::new());
	static ref REQUEST_ORIGINS: Mutex<HashMap<String, String>> = Mutex::new(HashMap::new());
	static ref MODULE_UUID_TO_ID: Mutex<HashMap<u128, String>> = Mutex::new(HashMap::new());
}

pub async fn handle_request(module_comm: &ModuleComm, data: String) {
	let json_result = serde_json::from_str(&data);

	if let Err(_) = json_result {
		return;
	}

	let input: Value = json_result.unwrap();

	let r#type = input[request_keys::TYPE].as_u64();
	let request_id = input[request_keys::REQUEST_ID].as_str();
	if r#type == None {
		send_error(module_comm, "undefined", errors::UNKNOWN_REQUEST).await;
		return;
	}
	let r#type = r#type.unwrap();
	if request_id == None {
		send_error(module_comm, "undefined", errors::INVALID_REQUEST_ID).await;
		return;
	}
	let request_id = request_id.unwrap();

	match r#type {
		request_types::MODULE_REGISTRATION => {
			handle_module_registration(module_comm, request_id, &input).await;
		}
		request_types::DECLARE_FUNCTION => {
			handle_declare_function(module_comm, request_id, &input).await;
		}
		request_types::FUNCTION_CALL => {
			handle_function_call(module_comm, request_id, &input).await;
		}
		request_types::FUNCTION_RESPONSE => {
			handle_function_response(module_comm, request_id, &input).await;
		}
		request_types::REGISTER_HOOK => {
			handle_register_hook(module_comm, request_id, &input).await;
		}
		request_types::TRIGGER_HOOK => {
			handle_trigger_hook(module_comm, request_id, &input).await;
		}
		_ => {
			send_error(module_comm, request_id, errors::UNKNOWN_REQUEST).await;
		}
	}
}

pub async fn on_module_disconnected(_module_comm: &ModuleComm) {
	// TODO recheck dependencies, hooks, registered modules, unregistered modules.
	/*
	let module_id = get_module_id_for_uuid(&module_comm.get_uuid()).await;

	if let None = module_id {
		return;
	}
	let module_id = module_id.unwrap();

	let mut registered_modules = REGISTERED_MODULES.lock().await;
	let mut unregistered_modules = UNREGISTERED_MODULES.lock().await;

	if registered_modules.contains_key(&module_id) {
		registered_modules
			.remove(&module_id)
			.unwrap()
			.close_sender()
			.await;
	} else if unregistered_modules.contains_key(&module_id) {
		unregistered_modules
			.remove(&module_id)
			.unwrap()
			.close_sender()
			.await;
	}
	*/
}

pub async fn is_uuid_exists(uuid: &u128) -> bool {
	MODULE_UUID_TO_ID.lock().await.contains_key(uuid)
}

#[allow(dead_code)]
async fn trigger_hook(
	module: &Module,
	hook: &str,
	data: &Map<String, Value>,
	sticky: bool,
	force: bool,
) {
	// module is trying to trigger a hook.
	// if force is true, all modules get the hook, regardless of whether they want it or not
	let module_id = module.get_module_id();
	let hook_name = module_id.clone() + "." + hook;

	for registered_module in REGISTERED_MODULES.lock().await.values() {
		if force || registered_module.is_hook_registered(&hook_name) {
			send_module(
				registered_module,
				&json!({
					request_keys::REQUEST_ID: generate_request_id().await,
					request_keys::TYPE: request_types::HOOK_TRIGGERED,
					request_keys::HOOK: hook_name,
					request_keys::DATA: data
				}),
			)
			.await;
		}
	}

	if sticky {
		// TODO sticky this hook somewhere so that new modules can get it
	}
}

async fn trigger_hook_on(from_module: &Module, to_module_id: &String, hook: &str, force: bool) {
	// from_module is trying to trigger a hook on to_module.
	// if force is true, all modules get the hook, regardless of whether they want it or not
	let module_id = from_module.get_module_id();
	let hook_name = module_id.clone() + "." + hook;
	let registered_modules = REGISTERED_MODULES.lock().await;
	let to_module = registered_modules.get(to_module_id);

	if let None = to_module {
		return;
	}

	let to_module = to_module.unwrap();

	if force || to_module.is_hook_registered(&hook_name) {
		send_module(
			to_module,
			&json!({
				request_keys::REQUEST_ID: generate_request_id().await,
				request_keys::TYPE: request_types::HOOK_TRIGGERED,
				request_keys::HOOK: hook_name
			}),
		)
		.await;
	}
}

async fn handle_module_registration(module_comm: &ModuleComm, request_id: &str, request: &Value) {
	let module_id = request[request_keys::MODULE_ID].as_str();
	let version = request[request_keys::VERSION].as_str();
	let dependencies = request[request_keys::DEPENDENCIES].as_object();

	if module_id == None {
		send_error(module_comm, request_id, errors::MALFORMED_REQUEST).await;
		return;
	}
	let module_id = module_id.unwrap();

	if version == None {
		send_error(module_comm, request_id, errors::MALFORMED_REQUEST).await;
		return;
	}
	let version = version.unwrap();

	let mut dependency_map = HashMap::new();

	// If dependencies are not null, then populate the dependency_map
	if let Some(dependencies) = dependencies {
		for dependency in dependencies.keys() {
			if !dependencies[dependency].is_string() {
				send_error(module_comm, request_id, errors::MALFORMED_REQUEST).await;
				return;
			}
			dependency_map.insert(
				dependency.clone(),
				String::from(dependencies[dependency].as_str().unwrap()),
			);
		}
	}

	let mut module = Module::new(
		module_comm.get_uuid().clone(),
		String::from(module_id),
		String::from(version),
		module_comm.clone_sender(),
	);
	module.set_dependencies(dependency_map);

	let mut registered_modules = REGISTERED_MODULES.lock().await;
	let mut unregistered_modules = UNREGISTERED_MODULES.lock().await;

	if registered_modules.contains_key(module_id) || unregistered_modules.contains_key(module_id) {
		send_error(module_comm, request_id, errors::DUPLICATE_MODULE).await;
		return;
	}

	// Register that this uuid belongs to this moduleId
	// check if this module_comm already has a corresponding module
	let mut module_uuid_to_id = MODULE_UUID_TO_ID.lock().await;
	if module_uuid_to_id.contains_key(module_comm.get_uuid()) {
		send_error(module_comm, request_id, errors::DUPLICATE_MODULE).await;
		return;
	}
	module_uuid_to_id.insert(module_comm.get_uuid().clone(), String::from(module_id));
	drop(module_uuid_to_id);

	send_module(
		&module,
		&json!({
			request_keys::REQUEST_ID: generate_request_id().await,
			request_keys::TYPE: request_types::MODULE_REGISTERED
		}),
	)
	.await;

	if module.get_dependencies().len() == 0 {
		module.set_registered(true);

		registered_modules.insert(String::from(module_id), module);
		drop(registered_modules);
		drop(unregistered_modules);

		trigger_hook_on(
			&module::GOTHAM_MODULE,
			&String::from(module_id),
			gotham_hooks::ACTIVATED,
			true,
		)
		.await;
	} else {
		module.set_registered(false);

		unregistered_modules.insert(String::from(module_id), module);
		drop(registered_modules);
		drop(unregistered_modules);
	}

	recalculate_all_module_dependencies().await;
}

async fn recalculate_all_module_dependencies() {
	// List of all modules whose dependencies weren't satisfied earlier but are satisfied now
	let mut satisfied_modules: Vec<String> = vec![];

	let mut registered_modules = REGISTERED_MODULES.lock().await;
	let mut unregistered_modules = UNREGISTERED_MODULES.lock().await;

	// recheck the dependencies for each unregistered module
	for (module_id, module) in unregistered_modules.iter() {
		// For each module, check if the dependencies are satisfied
		let mut dependency_satisfied = true;

		for (dependency, _) in module.get_dependencies() {
			// TODO check version as well
			if !registered_modules.contains_key(dependency)
				&& !unregistered_modules.contains_key(dependency)
			{
				dependency_satisfied = false;
				break;
			}
		}

		if dependency_satisfied {
			satisfied_modules.push(module_id.clone());
		}
	}

	// For all modules whose dependencies are now satisfied, register them
	for module_id in satisfied_modules {
		let module = unregistered_modules.remove(&module_id).unwrap();

		trigger_hook_on(
			&module::GOTHAM_MODULE,
			&module_id,
			gotham_hooks::ACTIVATED,
			true,
		)
		.await;
		registered_modules.insert(module_id, module);
	}
}

async fn handle_declare_function(module_comm: &ModuleComm, request_id: &str, request: &Value) {
	let module_id = get_module_id_for_uuid(&module_comm.get_uuid()).await;

	if let None = module_id {
		send_error(module_comm, request_id, errors::UNREGISTERED_MODULE).await;
		return;
	}
	let module_id = module_id.unwrap();

	let mut registered_modules = REGISTERED_MODULES.lock().await;

	// Check if module is registered
	if !registered_modules.contains_key(&module_id) {
		send_error(module_comm, request_id, errors::UNREGISTERED_MODULE).await;
		return;
	}

	let module = registered_modules.get_mut(&module_id).unwrap();
	let function = request[request_keys::FUNCTION].as_str();
	if let None = function {
		send_error(module_comm, request_id, errors::MALFORMED_REQUEST).await;
		return;
	}
	let function = function.unwrap();
	let function = String::from(function);
	if !module.is_function_declared(&function) {
		module.declare_function(function.clone());
	}

	send_module(
		module,
		&json!(
		{
			request_keys::REQUEST_ID: request_id,
			request_keys::TYPE: request_types::FUNCTION_DECLARED,
			request_keys::FUNCTION: function
		}),
	)
	.await;
}

async fn handle_function_call(module_comm: &ModuleComm, request_id: &str, request: &Value) {
	let module_id = get_module_id_for_uuid(&module_comm.get_uuid()).await;

	if let None = module_id {
		send_error(module_comm, request_id, errors::UNREGISTERED_MODULE).await;
		return;
	}
	let module_id = module_id.unwrap();

	// Check if module is registered
	if !REGISTERED_MODULES.lock().await.contains_key(&module_id) {
		send_error(module_comm, request_id, errors::UNREGISTERED_MODULE).await;
		return;
	}

	let function = request[request_keys::FUNCTION].as_str();

	if function == None {
		send_error(module_comm, request_id, errors::MALFORMED_REQUEST).await;
		return;
	}
	let function = function.unwrap();
	let function_name = is_function_name(function);

	if let None = function_name {
		send_error(module_comm, request_id, errors::UNKNOWN_FUNCTION).await;
		return;
	}
	let (module_name, function_name) = function_name.unwrap();

	let registered_modules = REGISTERED_MODULES.lock().await;
	if !registered_modules.contains_key(&module_name) {
		send_error(module_comm, request_id, errors::UNKNOWN_MODULE).await;
		return;
	}

	let receiver_module = registered_modules.get(&module_name).unwrap();
	if !receiver_module.is_function_declared(&function_name) {
		send_error(module_comm, request_id, errors::UNKNOWN_FUNCTION).await;
		return;
	}

	let mut request_origins = REQUEST_ORIGINS.lock().await;
	let request_id_heap = String::from(request_id);
	if request_origins.contains_key(&request_id_heap) {
		if request_origins[&request_id_heap] != module_id {
			// There's already a requestId that's supposed to return to
			// a different module. Let the module know that it's invalid
			// so that we can prevent response-hijacking.
			send_error(module_comm, request_id, errors::INVALID_REQUEST_ID).await;
			return;
		}
	} else {
		request_origins.insert(request_id_heap, module_id);
	}
	drop(request_origins);

	let mut response = request.clone();
	if request[request_keys::ARGUMENTS].as_object() == None {
		response[request_keys::ARGUMENTS] = Value::Object(Map::new());
	}

	response[request_keys::FUNCTION] = Value::String(function_name);

	send_module(receiver_module, &response).await;
}

async fn handle_function_response(module_comm: &ModuleComm, request_id: &str, request: &Value) {
	let module_id = get_module_id_for_uuid(&module_comm.get_uuid()).await;

	if let None = module_id {
		send_error(module_comm, request_id, errors::UNREGISTERED_MODULE).await;
		return;
	}
	let module_id = module_id.unwrap();

	// Check if module is registered
	let registered_modules = REGISTERED_MODULES.lock().await;
	if !registered_modules.contains_key(&module_id) {
		send_error(module_comm, request_id, errors::UNREGISTERED_MODULE).await;
		return;
	}

	let request_origins = REQUEST_ORIGINS.lock().await;

	if !request_origins.contains_key(request_id) {
		// If the given requestId does not contain an origin,
		// drop the packet entirely
		return;
	}

	let origin_module_id = request_origins.get(&String::from(request_id)).unwrap();

	if !registered_modules.contains_key(origin_module_id) {
		// The origin module has probably disconnected.
		// Drop the packet entirely
		return;
	}

	send_module(registered_modules.get(origin_module_id).unwrap(), request).await;
}

async fn handle_register_hook(module_comm: &ModuleComm, request_id: &str, request: &Value) {
	// The module who is calling this function wants to listen for a hook
	let module_id = get_module_id_for_uuid(&module_comm.get_uuid()).await;

	if let None = module_id {
		send_error(module_comm, request_id, errors::UNREGISTERED_MODULE).await;
		return;
	}
	let module_id = module_id.unwrap();

	let mut registered_modules = REGISTERED_MODULES.lock().await;

	let module = registered_modules.get_mut(&module_id);
	if let None = module {
		send_error(module_comm, request_id, errors::UNREGISTERED_MODULE).await;
		return;
	}
	let module = module.unwrap();

	let hook = request[request_keys::HOOK].as_str();
	if hook == None {
		send_error(module_comm, request_id, errors::MALFORMED_REQUEST).await;
		return;
	}
	let hook = String::from(hook.unwrap());

	if !module.is_hook_registered(&hook) {
		module.register_hook(hook);
	}

	send_module(
		module,
		&json!(
		{
			request_keys::REQUEST_ID: request_id,
			request_keys::TYPE: request_types::HOOK_REGISTERED
		}),
	)
	.await;
}

async fn handle_trigger_hook(module_comm: &ModuleComm, request_id: &str, request: &Value) {
	// The module who is calling this function is triggering a hook
	let module_id = get_module_id_for_uuid(&module_comm.get_uuid()).await;

	if let None = module_id {
		send_error(module_comm, request_id, errors::UNREGISTERED_MODULE).await;
		return;
	}
	let module_id = module_id.unwrap();

	let mut registered_modules = REGISTERED_MODULES.lock().await;

	let module = registered_modules.get_mut(&module_id);
	if let None = module {
		send_error(module_comm, request_id, errors::UNREGISTERED_MODULE).await;
		return;
	}
	let module = module.unwrap();

	let hook = request[request_keys::HOOK].as_str();
	if hook == None {
		send_error(module_comm, request_id, errors::MALFORMED_REQUEST).await;
		return;
	}
	let hook = hook.unwrap();

	let data = request[request_keys::DATA].as_object();
	let data = if data == None {
		Map::new()
	} else {
		data.unwrap().clone()
	};

	trigger_hook(&module, &hook, &data, false, false).await;

	send_module(
		module,
		&json!(
		{
			request_keys::REQUEST_ID: request_id,
			request_keys::TYPE: request_types::HOOK_TRIGGERED
		}),
	)
	.await;
}

fn is_function_name(name: &str) -> Option<(String, String)> {
	if !name.contains(".") {
		return None;
	}

	let parts: Vec<&str> = name.split(".").collect();

	if parts.len() != 2 {
		return None;
	}

	for letter in parts[0].chars() {
		if !letter.is_alphanumeric() && letter != '-' && letter != '_' {
			return None;
		}
	}

	for letter in parts[1].chars() {
		if !letter.is_alphanumeric() && letter != '_' {
			return None;
		}
	}

	Some((String::from(parts[0]), String::from(parts[1])))
}

async fn get_module_id_for_uuid(module_uuid: &u128) -> Option<String> {
	let module_uuid_to_id = MODULE_UUID_TO_ID.lock().await;
	let module_id = module_uuid_to_id.get(module_uuid)?;
	Some(module_id.clone())
}

async fn generate_request_id() -> String {
	String::from(format!("gotham{}", get_current_nanos()))
}

fn get_current_nanos() -> u128 {
	SystemTime::now()
		.duration_since(UNIX_EPOCH)
		.expect("Time went backwards. Wtf?")
		.as_nanos()
}

async fn send_error(module_comm: &ModuleComm, request_id: &str, error_code: u32) {
	send_module_comm(
		module_comm,
		&json!({
			request_keys::REQUEST_ID: request_id,
			request_keys::TYPE: request_types::ERROR,
			request_keys::ERROR: error_code
		}),
	)
	.await;
}

async fn send_module_comm(module_comm: &ModuleComm, data: &Value) {
	module_comm.send(data.to_string() + "\n").await;
}

async fn send_module(module: &Module, data: &Value) {
	module.send(data.to_string() + "\n").await;
}
