use crate::{
	models::{Module, ModuleComm},
	utils::{
		constants::{self, errors, juno_hooks, request_keys, request_types},
		logger,
	},
};

use std::{
	collections::HashMap,
	time::{SystemTime, UNIX_EPOCH},
};

use async_std::sync::Mutex;

use serde_json::{json, Map, Value};

use semver::{Version, VersionReq};

lazy_static! {
	static ref REGISTERED_MODULES: Mutex<HashMap<String, Module>> = Mutex::new(HashMap::new());
	static ref UNREGISTERED_MODULES: Mutex<HashMap<String, Module>> = Mutex::new(HashMap::new());
	static ref REQUEST_ORIGINS: Mutex<HashMap<String, String>> = Mutex::new(HashMap::new());
	static ref MODULE_UUID_TO_ID: Mutex<HashMap<u128, String>> = Mutex::new(HashMap::new());
}

pub async fn handle_request(module_comm: &ModuleComm, data: String) {
	let json_result = serde_json::from_str(&data);
	logger::verbose("Got request. Processing...");

	if json_result.is_err() {
		logger::warn("Request is not parsable. Ignoring...");
		return;
	}

	let input: Value = json_result.unwrap();

	let r#type = input[request_keys::TYPE].as_u64();
	let request_id = input[request_keys::REQUEST_ID].as_str();
	if r#type == None {
		logger::warn("type not present. Sending error...");
		send_error(module_comm, "undefined", errors::UNKNOWN_REQUEST).await;
		return;
	}
	let r#type = r#type.unwrap();
	if request_id == None {
		logger::warn("requestId not present. Sending error...");
		send_error(module_comm, "undefined", errors::INVALID_REQUEST_ID).await;
		return;
	}
	let request_id = request_id.unwrap();

	match r#type {
		request_types::REGISTER_MODULE_REQUEST => {
			logger::verbose("Processing request as module registration...");
			handle_module_registration(module_comm, request_id, &input).await;
		}
		request_types::DECLARE_FUNCTION_REQUEST => {
			logger::verbose("Processing request as declare function...");
			handle_declare_function(module_comm, request_id, &input).await;
		}
		request_types::FUNCTION_CALL_REQUEST => {
			logger::verbose("Processing request as function call...");
			handle_function_call(module_comm, request_id, &input).await;
		}
		request_types::FUNCTION_CALL_RESPONSE => {
			logger::verbose("Processing request as function response...");
			handle_function_response(module_comm, request_id, &input).await;
		}
		request_types::REGISTER_HOOK_REQUEST => {
			logger::verbose("Processing request as register hook...");
			handle_register_hook(module_comm, request_id, &input).await;
		}
		request_types::TRIGGER_HOOK_REQUEST => {
			logger::verbose("Processing request as trigger hook...");
			handle_trigger_hook(module_comm, request_id, &input).await;
		}
		_ => {
			logger::debug(&format!(
				"Found unknown request type {}. Sending error...",
				r#type
			));
			send_error(module_comm, request_id, errors::UNKNOWN_REQUEST).await;
		}
	}
	logger::verbose("Completed processing the request");
}

pub async fn get_registered_modules() -> Vec<Module> {
	let registered_modules = REGISTERED_MODULES.lock().await;
	let mut modules = vec![];

	for module in registered_modules.values() {
		modules.push(module.clone());
	}

	modules
}

pub async fn get_unregistered_modules() -> Vec<Module> {
	let unregistered_modules = UNREGISTERED_MODULES.lock().await;
	let mut modules = vec![];

	for module in unregistered_modules.values() {
		modules.push(module.clone());
	}

	modules
}

pub async fn get_module_by_id(module_id: &str) -> Option<Module> {
	let registered_modules = REGISTERED_MODULES.lock().await;
	if registered_modules.contains_key(module_id) {
		return Some(registered_modules.get(module_id).unwrap().clone());
	}
	drop(registered_modules);

	let unregistered_modules = UNREGISTERED_MODULES.lock().await;
	if unregistered_modules.contains_key(module_id) {
		return Some(unregistered_modules.get(module_id).unwrap().clone());
	}
	drop(unregistered_modules);

	None
}

async fn handle_module_registration(module_comm: &ModuleComm, request_id: &str, request: &Value) {
	let module_id = request[request_keys::MODULE_ID].as_str();
	let version = request[request_keys::VERSION].as_str();
	let dependencies = request[request_keys::DEPENDENCIES].as_object();

	if module_id == None {
		logger::debug("moduleId not present. Sending error...");
		send_error(module_comm, request_id, errors::MALFORMED_REQUEST).await;
		return;
	}
	let module_id = module_id.unwrap();

	if version == None {
		logger::debug("version not present. Sending error...");
		send_error(module_comm, request_id, errors::MALFORMED_REQUEST).await;
		return;
	}
	let version = version.unwrap();

	let mut dependency_map = HashMap::new();

	// If dependencies are not null, then populate the dependency_map
	if let Some(dependencies) = dependencies {
		logger::verbose("Dependency is not null. Populating hashmap...");
		for dependency in dependencies.keys() {
			if !dependencies[dependency].is_string() {
				logger::debug(&format!(
					"Dependency value for key {} is not a string. Sending error...",
					dependency
				));
				send_error(module_comm, request_id, errors::MALFORMED_REQUEST).await;
				return;
			}
			let dependency_requirement =
				VersionReq::parse(dependencies[dependency].as_str().unwrap());
			if dependency_requirement.is_err() {
				logger::debug(&format!("Dependency value for key {} is not a valid SemVer version requirement. Sending error...", dependency));
				send_error(module_comm, request_id, errors::MALFORMED_REQUEST).await;
				return;
			}
			dependency_map.insert(dependency.clone(), dependency_requirement.unwrap());
		}
		logger::verbose(&format!(
			"HashMap populated with {} dependencies",
			dependency_map.len()
		));
	} else {
		logger::verbose("Dependency is null. Pre-assigning a new HashMap");
	}

	let version = Version::parse(version);
	if version.is_err() {
		logger::debug("version not valid. Sending error...");
		send_error(module_comm, request_id, errors::MALFORMED_REQUEST).await;
		return;
	}
	let version = version.unwrap();

	let mut module = Module::new(
		*module_comm.get_uuid(),
		String::from(module_id),
		version,
		module_comm.clone_sender(),
	);
	module.set_dependencies(dependency_map);

	let mut registered_modules = REGISTERED_MODULES.lock().await;
	let mut unregistered_modules = UNREGISTERED_MODULES.lock().await;

	if registered_modules.contains_key(module_id) || unregistered_modules.contains_key(module_id) {
		logger::debug("Either registered modules or unregistered modules already has this moduleId. Sending error...");
		send_error(module_comm, request_id, errors::DUPLICATE_MODULE).await;
		return;
	}

	// Register that this uuid belongs to this moduleId
	// check if this module_comm already has a corresponding module
	let mut module_uuid_to_id = MODULE_UUID_TO_ID.lock().await;
	if module_uuid_to_id.contains_key(module_comm.get_uuid()) {
		logger::debug("A moduleId for that UUID already exists. This looks like a duplicate module. Sending error...");
		send_error(module_comm, request_id, errors::DUPLICATE_MODULE).await;
		return;
	}
	module_uuid_to_id.insert(*module_comm.get_uuid(), String::from(module_id));
	drop(module_uuid_to_id);

	logger::verbose("Notifying successful module registration...");
	send_module(
		&module,
		&json!({
			request_keys::REQUEST_ID: request_id,
			request_keys::TYPE: request_types::REGISTER_MODULE_RESPONSE
		}),
	)
	.await;
	logger::verbose("Notification successful");

	if module.get_dependencies().is_empty() {
		module.set_registered(true);

		logger::verbose("Triggering activation hook...");
		trigger_hook_on(
			constants::APP_NAME,
			&module,
			juno_hooks::ACTIVATED,
			&Map::new(),
			true,
		)
		.await;
		logger::verbose("Activation hook triggered");

		logger::verbose("Dependencies are none. Adding to registered modules...");
		registered_modules.insert(String::from(module_id), module);
		logger::verbose("Module added to registered modules");
	} else {
		module.set_registered(false);

		logger::verbose("Module has dependencies. Adding to unregistered modules...");
		unregistered_modules.insert(String::from(module_id), module);
		logger::verbose("Module added to unregistered modules");
	}
	drop(registered_modules);
	drop(unregistered_modules);

	recalculate_all_module_dependencies().await;
}

async fn handle_declare_function(module_comm: &ModuleComm, request_id: &str, request: &Value) {
	let module_id = get_module_id_for_uuid(&module_comm.get_uuid()).await;

	if module_id.is_none() {
		logger::debug("moduleId not found. Sending error...");
		send_error(module_comm, request_id, errors::UNREGISTERED_MODULE).await;
		return;
	}
	let module_id = module_id.unwrap();

	let mut registered_modules = REGISTERED_MODULES.lock().await;

	// Check if module is registered
	if !registered_modules.contains_key(&module_id) {
		logger::debug("This module is not registered. Sending error...");
		send_error(module_comm, request_id, errors::UNREGISTERED_MODULE).await;
		return;
	}

	let module = registered_modules.get_mut(&module_id).unwrap();
	let function = request[request_keys::FUNCTION].as_str();
	if function.is_none() {
		logger::debug("Function is not parsable as a string. Sending error...");
		send_error(module_comm, request_id, errors::MALFORMED_REQUEST).await;
		return;
	}
	let function = function.unwrap();
	let function = String::from(function);
	if !module.is_function_declared(&function) {
		module.declare_function(function.clone());
		logger::info(&format!(
			"Function '{}' declared on module '{}'",
			function, module_id
		));
	} else {
		logger::warn("This function is already declared. No need to register it again");
	}

	logger::verbose("Informing module of successful function declaration...");
	send_module(
		module,
		&json!(
		{
			request_keys::REQUEST_ID: request_id,
			request_keys::TYPE: request_types::DECLARE_FUNCTION_RESPONSE,
			request_keys::FUNCTION: function
		}),
	)
	.await;
	logger::verbose("Success response has been sent");
}

async fn handle_function_call(module_comm: &ModuleComm, request_id: &str, request: &Value) {
	let module_id = get_module_id_for_uuid(&module_comm.get_uuid()).await;

	if module_id.is_none() {
		logger::debug("moduleId not found. Sending error...");
		send_error(module_comm, request_id, errors::UNREGISTERED_MODULE).await;
		return;
	}
	let module_id = module_id.unwrap();

	// Check if module is registered
	if !REGISTERED_MODULES.lock().await.contains_key(&module_id) {
		logger::debug("This module is not registered. Sending error...");
		send_error(module_comm, request_id, errors::UNREGISTERED_MODULE).await;
		return;
	}

	let function = request[request_keys::FUNCTION].as_str();

	if function == None {
		logger::debug("Function is not parsable as a string. Sending error...");
		send_error(module_comm, request_id, errors::MALFORMED_REQUEST).await;
		return;
	}
	let function = function.unwrap();
	logger::verbose(&format!("Got request to call function '{}'", function));
	let function_name = is_function_name(function);

	if function_name.is_none() {
		logger::debug(
			"Function is not of the format 'module-name.function_name'. Sending error...",
		);
		send_error(module_comm, request_id, errors::UNKNOWN_FUNCTION).await;
		return;
	}
	let (module_name, function_name) = function_name.unwrap();
	logger::info(&format!(
		"Calling function '{}' in module '{}'",
		function_name, module_name
	));

	let registered_modules = REGISTERED_MODULES.lock().await;
	if !registered_modules.contains_key(&module_name) {
		logger::debug(&format!(
			"The module '{}' is not registered. Sending error...",
			module_name
		));
		send_error(module_comm, request_id, errors::UNKNOWN_MODULE).await;
		return;
	}

	let receiver_module = registered_modules.get(&module_name).unwrap();
	if !receiver_module.is_function_declared(&function_name) {
		logger::debug(&format!(
			"The function '{}' is not declared. Sending error...",
			function_name
		));
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
			logger::error(&format!("The call to function '{}' had a requestId '{}', which is already declared. In order to prevent request hijacking, this request will be errored. Sending error...", function, request_id));
			send_error(module_comm, request_id, errors::INVALID_REQUEST_ID).await;
			return;
		} else {
			logger::debug(&format!("There already seems to be a request to module '{}' with the requestId '{}'. This may or may not be intended. Are you sending the same request twice?", module_name, request_id));
		}
	} else {
		logger::verbose("Registering the requestId along with it's origin module.");
		request_origins.insert(request_id_heap, module_id);
	}
	drop(request_origins);

	let mut response = request.clone();
	logger::verbose("Cloning request to send as response");
	if request[request_keys::ARGUMENTS].as_object() == None {
		logger::debug("The call to function had no arguments (or the arguments were not an object). A new, empty object will be assigned");
		response[request_keys::ARGUMENTS] = Value::Object(Map::new());
	}

	logger::verbose("Changing the value of function from 'module-name.function_name' to just 'function_name' in the request");
	response[request_keys::FUNCTION] = Value::String(function_name);

	logger::verbose("Proxying the request to the relevant module...");
	send_module(receiver_module, &response).await;
	logger::verbose("Function call proxied.");
}

async fn handle_function_response(module_comm: &ModuleComm, request_id: &str, request: &Value) {
	let module_id = get_module_id_for_uuid(&module_comm.get_uuid()).await;
	let request_id = &String::from(request_id);

	if module_id.is_none() {
		logger::debug("moduleId not found. Sending error...");
		send_error(module_comm, request_id, errors::UNREGISTERED_MODULE).await;
		return;
	}
	let module_id = module_id.unwrap();

	// Check if module is registered
	let registered_modules = REGISTERED_MODULES.lock().await;
	if !registered_modules.contains_key(&module_id) {
		logger::debug(&format!(
			"The module '{}' is not registered. Sending error...",
			module_id
		));
		send_error(module_comm, request_id, errors::UNREGISTERED_MODULE).await;
		return;
	}

	let mut request_origins = REQUEST_ORIGINS.lock().await;

	if !request_origins.contains_key(request_id) {
		// If the given requestId does not contain an origin,
		// drop the packet entirely
		logger::error(&format!("The function response with requestId '{}' does not contain an origin. The response might be malformed. Please ensure the function response has the same requestId as the function call", request_id));
		send_error(module_comm, request_id, errors::MALFORMED_REQUEST).await;
		return;
	}

	// This requestId has completed its round-trip. Remove it from request_origins so that we can add the same one later on
	let origin_module_id = request_origins.remove(request_id).unwrap();

	if !registered_modules.contains_key(&origin_module_id) {
		// The origin module has probably disconnected.
		// Drop the packet entirely
		logger::debug(&format!("The function response meant for module '{}' is not registered (is the module still connected?). This packet will be ignored.", origin_module_id));
		return;
	}

	logger::verbose("Sending back response to origin module...");
	send_module(registered_modules.get(&origin_module_id).unwrap(), request).await;
	logger::verbose("Function response to origin module successfully sent.");
}

async fn handle_register_hook(module_comm: &ModuleComm, request_id: &str, request: &Value) {
	// The module who is calling this function wants to listen for a hook
	let module_id = get_module_id_for_uuid(&module_comm.get_uuid()).await;

	if module_id.is_none() {
		logger::debug("moduleId not found. Sending error...");
		send_error(module_comm, request_id, errors::UNREGISTERED_MODULE).await;
		return;
	}
	let module_id = module_id.unwrap();

	let mut registered_modules = REGISTERED_MODULES.lock().await;

	let module = registered_modules.get_mut(&module_id);
	if module.is_none() {
		logger::debug("This module is not registered. Sending error...");
		send_error(module_comm, request_id, errors::UNREGISTERED_MODULE).await;
		return;
	}
	let module = module.unwrap();

	let hook = request[request_keys::HOOK].as_str();
	if hook == None {
		logger::debug("Hook is not parsable as a string. Sending error...");
		send_error(module_comm, request_id, errors::MALFORMED_REQUEST).await;
		return;
	}
	let hook = String::from(hook.unwrap());

	if !module.is_hook_registered(&hook) {
		logger::verbose(&format!(
			"The hook '{}' is not registered. Registering hook...",
			hook
		));
		module.register_hook(hook);
	} else {
		logger::debug(&format!(
			"The hook '{}' is already registered. No need to register again.",
			hook
		));
	}

	logger::verbose("Hook registered. Sending success response to module...");
	send_module(
		module,
		&json!(
		{
			request_keys::REQUEST_ID: request_id,
			request_keys::TYPE: request_types::REGISTER_HOOK_RESPONSE
		}),
	)
	.await;
	logger::info("Hook registration done, and success response has been sent");
}

async fn handle_trigger_hook(module_comm: &ModuleComm, request_id: &str, request: &Value) {
	// The module who is calling this function is triggering a hook
	let module_id = get_module_id_for_uuid(&module_comm.get_uuid()).await;

	if module_id.is_none() {
		logger::debug("moduleId not found. Sending error...");
		send_error(module_comm, request_id, errors::UNREGISTERED_MODULE).await;
		return;
	}
	let module_id = module_id.unwrap();

	let registered_modules = REGISTERED_MODULES.lock().await;

	let module = registered_modules.get(&module_id);
	if module.is_none() {
		logger::debug("This module is not registered. Sending error...");
		send_error(module_comm, request_id, errors::UNREGISTERED_MODULE).await;
		return;
	}
	let module = module.unwrap();

	let hook = request[request_keys::HOOK].as_str();
	if hook == None {
		logger::debug("Hook is not parsable as a string. Sending error...");
		send_error(module_comm, request_id, errors::MALFORMED_REQUEST).await;
		return;
	}
	let hook = hook.unwrap();

	let data = request[request_keys::DATA].as_object();
	let data = if data == None {
		logger::debug("The triggered hook had no arguments (or the arguments were not an object). A new, empty object will be assigned");
		Map::new()
	} else {
		logger::verbose("The hook data will be cloned to send to modules");
		data.unwrap().clone()
	};

	logger::verbose(&format!("Triggering hook '{}' on all modules...", hook));
	trigger_hook(&module, &hook, &data, false, false).await;

	logger::verbose(
		"Hook triggered on all modules. Informing origin module of successful hook trigger...",
	);
	send_module(
		module,
		&json!(
		{
			request_keys::REQUEST_ID: request_id,
			request_keys::TYPE: request_types::TRIGGER_HOOK_RESPONSE
		}),
	)
	.await;
	logger::verbose("Origin module has been notified of hook triggered");
}

pub async fn on_module_disconnected(module_comm: &ModuleComm) {
	logger::info(&format!(
		"Module with UUID {} disconnected. Processing...",
		module_comm.get_uuid()
	));
	// recheck dependencies
	let module_id = get_module_id_for_uuid(&module_comm.get_uuid()).await;

	if module_id.is_none() {
		logger::verbose("Module does not have a moduleId. No more processing required");
		return;
	}
	let module_id = module_id.unwrap();
	logger::verbose(&format!(
		"Module is associated with moduleId '{}'",
		module_id
	));

	let mut registered_modules = REGISTERED_MODULES.lock().await;
	let mut unregistered_modules = UNREGISTERED_MODULES.lock().await;

	if registered_modules.contains_key(&module_id) {
		logger::verbose("Module is a registered module. Removing...");
		registered_modules
			.remove(&module_id)
			.unwrap()
			.close_sender()
			.await;
		logger::verbose("Module removed from registered modules");
	} else if unregistered_modules.contains_key(&module_id) {
		logger::verbose("Module is an registered module. Removing...");
		unregistered_modules
			.remove(&module_id)
			.unwrap()
			.close_sender()
			.await;
		logger::verbose("Module removed from unregistered modules");
	}
	drop(registered_modules);
	drop(unregistered_modules);

	recalculate_all_module_dependencies().await;
	logger::verbose("Module is no longer tracked");
}

pub async fn is_uuid_exists(uuid: &u128) -> bool {
	MODULE_UUID_TO_ID.lock().await.contains_key(uuid)
}

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
	logger::info(&format!(
		"Module '{}' is triggering the hook '{}' on all modules",
		module_id, hook_name
	));

	logger::verbose("Iterating all registered modules to send hook to...");
	for registered_module in REGISTERED_MODULES.lock().await.values() {
		if force {
			logger::verbose(&format!(
				"Hook is being forced onto module '{}'...",
				registered_module.get_module_id()
			));
			send_module(
				registered_module,
				&json!({
					request_keys::REQUEST_ID: generate_request_id().await,
					request_keys::TYPE: request_types::TRIGGER_HOOK_RESPONSE,
					request_keys::HOOK: hook_name,
					request_keys::DATA: data
				}),
			)
			.await;
		} else if registered_module.is_hook_registered(&hook_name) {
			logger::verbose(&format!(
				"Module '{}' is listening for this hook. Sending hook to module...",
				registered_module.get_module_id()
			));
			send_module(
				registered_module,
				&json!({
					request_keys::REQUEST_ID: generate_request_id().await,
					request_keys::TYPE: request_types::TRIGGER_HOOK_RESPONSE,
					request_keys::HOOK: hook_name,
					request_keys::DATA: data
				}),
			)
			.await;
		} else {
			logger::verbose(&format!(
				"Module '{}' is not listening for this hook. Hook is not being sent to module",
				registered_module.get_module_id()
			));
		}
	}
	logger::verbose("All registered modules have been processed");

	if sticky {
		logger::info("This hook is being stickied. Saving it for future modules...");
		logger::warn("TODO stickying hooks is not implemented yet");
	} else {
		logger::verbose("This hook is not being stickied.");
	}
}

async fn trigger_hook_on(
	from_module: &str,
	to_module: &Module,
	hook: &str,
	data: &Map<String, Value>,
	force: bool,
) {
	// from_module is trying to trigger a hook on to_module.
	// if force is true, all modules get the hook, regardless of whether they want it or not
	let module_id = String::from(from_module);
	let hook_name = module_id.clone() + "." + hook;
	logger::info(&format!(
		"Triggering '{}' hook on module '{}'...",
		hook_name,
		to_module.get_module_id()
	));

	if force {
		logger::verbose("Hook is being forced onto the module...");
		send_module(
			to_module,
			&json!({
				request_keys::REQUEST_ID: generate_request_id().await,
				request_keys::TYPE: request_types::TRIGGER_HOOK_RESPONSE,
				request_keys::HOOK: hook_name,
				request_keys::DATA: data
			}),
		)
		.await;
	} else if to_module.is_hook_registered(&hook_name) {
		logger::verbose("The module is registered for the hook. Sending hook...");
		send_module(
			to_module,
			&json!({
				request_keys::REQUEST_ID: generate_request_id().await,
				request_keys::TYPE: request_types::TRIGGER_HOOK_RESPONSE,
				request_keys::HOOK: hook_name,
				request_keys::DATA: data
			}),
		)
		.await;
	}
	logger::verbose("Hook sent to module.");
}

async fn recalculate_all_module_dependencies() {
	logger::verbose("Recalculating all module dependencies...");
	// List of all modules whose dependencies weren't satisfied earlier but are satisfied now
	let mut satisfied_modules: Vec<String> = vec![];

	let mut registered_modules = REGISTERED_MODULES.lock().await;
	let mut unregistered_modules = UNREGISTERED_MODULES.lock().await;

	logger::verbose("Checking if any unregistered modules are satisfied...");
	// recheck the dependencies for each unregistered module
	for (module_id, module) in unregistered_modules.iter() {
		// For each module, check if the dependencies are satisfied
		let mut dependency_satisfied = true;

		logger::verbose(&format!("Checking dependencies for module '{}'", module_id));
		for (dependency, version_req) in module.get_dependencies() {
			if registered_modules.contains_key(dependency) {
				if !version_req.matches(&registered_modules.get(dependency).unwrap().get_version())
				{
					logger::debug(&format!(
						"Dependency '{}' is incompatible. Required '{}', present '{}'",
						dependency,
						version_req.to_string(),
						registered_modules.get(dependency).unwrap().get_version()
					));
					dependency_satisfied = false;
					break;
				}
			} else if unregistered_modules.contains_key(dependency) {
				if !version_req
					.matches(&unregistered_modules.get(dependency).unwrap().get_version())
				{
					logger::debug(&format!(
						"Dependency '{}' is incompatible. Required '{}', present '{}'",
						dependency,
						version_req.to_string(),
						registered_modules.get(dependency).unwrap().get_version()
					));
					dependency_satisfied = false;
					break;
				}
			} else {
				logger::debug(&format!(
					"Dependency '{}' not present. Can't register this module",
					dependency
				));
				dependency_satisfied = false;
				break;
			}
		}

		if dependency_satisfied {
			logger::verbose(&format!(
				"All dependencies for moduleId '{}' are satisfied.",
				module_id
			));
			satisfied_modules.push(module_id.clone());
		}
	}

	logger::verbose("Registering all satisfied modules...");
	// For all modules whose dependencies are now satisfied, register them
	for module_id in satisfied_modules {
		logger::verbose(&format!(
			"Module {} is now satisfied. Registering...",
			module_id
		));
		let mut module = unregistered_modules.remove(&module_id).unwrap();
		module.set_registered(true);

		logger::verbose("Sending ACTIVATED trigger to module...");
		trigger_hook_on(
			constants::APP_NAME,
			&module,
			juno_hooks::ACTIVATED,
			&Map::new(),
			true,
		)
		.await;
		logger::verbose("ACTIVATED trigger sent.");

		logger::verbose("Adding module to registered_modules...");
		registered_modules.insert(module_id, module);
		logger::verbose("Module registered");
	}
	logger::verbose("All newly satisfied modules registered");

	// List of all modules whose dependencies were satisfied but aren't now
	let mut unsatisfied_modules: Vec<String> = vec![];

	logger::verbose("Checking if any registered modules are no longer satisfied...");
	// remove modules whose dependencies are no longer satisfied
	for (module_id, module) in registered_modules.iter() {
		// For each module, check if the dependencies are satisfied
		let mut dependency_satisfied = true;

		logger::verbose(&format!("Checking dependencies for module '{}'", module_id));
		for (dependency, version_req) in module.get_dependencies() {
			if registered_modules.contains_key(dependency) {
				if !version_req.matches(&registered_modules.get(dependency).unwrap().get_version())
				{
					logger::debug(&format!(
						"Dependency '{}' is incompatible. Required '{}', present '{}'",
						dependency,
						version_req.to_string(),
						registered_modules.get(dependency).unwrap().get_version()
					));
					dependency_satisfied = false;
					break;
				}
			} else {
				logger::debug(&format!(
					"Dependency '{}' not present or not registered. Can't keep this module registered",
					dependency
				));
				dependency_satisfied = false;
				break;
			}
		}

		if !dependency_satisfied {
			logger::verbose(&format!(
				"Not all dependencies for moduleId '{}' are satisfied. This module will be unregistered",
				module_id
			));
			unsatisfied_modules.push(module_id.clone());
		}
	}

	logger::verbose("Unregistering all modules that are no longer satisfied...");
	// For all modules whose dependencies are no longer satisfied, unregister them
	for module_id in unsatisfied_modules {
		logger::verbose(&format!(
			"Module {} is no longer satisfied. Unregistering...",
			module_id
		));
		let mut module = registered_modules.remove(&module_id).unwrap();
		module.set_registered(false);

		logger::verbose("Sending DEACTIVATED trigger to module...");
		trigger_hook_on(
			constants::APP_NAME,
			&module,
			juno_hooks::DEACTIVATED,
			&Map::new(),
			true,
		)
		.await;
		logger::verbose("DEACTIVATED trigger sent");

		logger::verbose("Adding module to unregistered_modules...");
		unregistered_modules.insert(module_id, module);
		logger::verbose("Module unregistered");
	}
	logger::verbose("All module whose dependencies are no longer satisfied are unregistered");

	logger::verbose("All module dependencies recalculated");
}

fn is_function_name(name: &str) -> Option<(String, String)> {
	if !name.contains('.') {
		return None;
	}

	let parts: Vec<&str> = name.split('.').collect();

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
	format!("juno{}", get_current_nanos())
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
	let error_name = match error_code {
		errors::DUPLICATE_MODULE => "DUPLICATE_MODULE",
		errors::INVALID_MODULE_ID => "INVALID_MODULE_ID",
		errors::INVALID_REQUEST_ID => "INVALID_REQUEST_ID",
		errors::MALFORMED_REQUEST => "MALFORMED_REQUEST",
		errors::UNKNOWN_FUNCTION => "UNKNOWN_FUNCTION",
		errors::UNKNOWN_MODULE => "UNKNOWN_MODULE",
		errors::UNKNOWN_REQUEST => "UNKNOWN_REQUEST",
		errors::UNREGISTERED_MODULE => "UNREGISTERED_MODULE",
		_ => "undefined",
	};
	logger::verbose(&format!("{} error sent", error_name));
}

async fn send_module_comm(module_comm: &ModuleComm, data: &Value) {
	module_comm.send(data.to_string() + "\n").await;
}

async fn send_module(module: &Module, data: &Value) {
	module.send(data.to_string() + "\n").await;
}
