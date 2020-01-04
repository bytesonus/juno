use crate::models::Module;
use crate::models::ModuleComm;
use crate::utils::constants::request_keys;

use std::collections::HashMap;

use async_std::sync::Mutex;

use serde_json::{json, to_string, to_string_pretty, Value};

lazy_static! {
	static ref REGISTERED_MODULES: Mutex<HashMap<String, Module>> = Mutex::new(HashMap::new());
	static ref UNREGISTERED_MODULES: Mutex<HashMap<String, Module>> = Mutex::new(HashMap::new());
	static ref REQUEST_ORIGINS: Mutex<HashMap<String, String>> = Mutex::new(HashMap::new());
}

pub async fn handle_request(module_comm: &ModuleComm, data: String) {
	let json_result = serde_json::from_str(&data);

	if let Err(_) = json_result {
		return;
	}

	let input: Value = json_result.unwrap();

	let r#type = input[request_keys::TYPE].as_str();
	let request_id = input[request_keys::REQUEST_ID].as_str();
	if r#type == None {
		// SendError(unknownRequest);
		return;
	}
	let r#type = r#type.unwrap();
	if request_id == None {
		// SendError(invalidRequestId);
		return;
	}
	let request_id = request_id.unwrap();

	match r#type {
		"moduleRegistration" => {
			handle_module_registration(module_comm, request_id, &input).await;
		}
		_ => {
			// SendError(unknownRequest);
		}
	}
}

async fn handle_module_registration(module_comm: &ModuleComm, request_id: &str, request: &Value) {
	let module_id = request[request_keys::MODULE_ID].as_str();
	let version = request[request_keys::VERSION].as_str();

	if module_id == None {
		// SendError(malformedRequest);
		return;
	}
	let module_id = module_id.unwrap();

	if version == None {
		// SendError(malformedRequest);
		return;
	}
	let version = version.unwrap();

	let registered_modules = REGISTERED_MODULES.lock().await;
	let mut unregistered_modules = UNREGISTERED_MODULES.lock().await;

	if registered_modules.contains_key(module_id) || unregistered_modules.contains_key(module_id) {
		// SendError(invalidModuleId)
		return;
	}

	let module = Module::new(
		module_comm.get_uuid(),
		String::from(module_id),
		String::from(version),
		module_comm.clone_sender(),
	);

	unregistered_modules.insert(String::from(module_id), module);

	module_comm
		.send(format!(
			"{}",
			json!({
				request_keys::REQUEST_ID: request_id,
				request_keys::TYPE: "moduleRegistered"
			})
		))
		.await;
}

/*
internal static Dictionary<string, Module> UnRegisteredModules = new Dictionary<string, Module>();
internal static Dictionary<string, string> RequestOrigins = new Dictionary<string, string>();
private static Dictionary<string, ModuleRequestHandler> RequestHandlers = new Dictionary<string, ModuleRequestHandler>
{
	{
		Constants.ModuleRegistration,
		new ModuleRequestHandler(HandleModuleRegistration)
		{
			ParametersRequired =
			{
				new ModuleRequestParameter(Constants.ModuleId),
				new ModuleRequestParameter(Constants.Version),
				new ModuleRequestParameter(Constants.Dependencies)
				{
					ParameterType = JTokenType.Object,
					Optional = true
				}
			}
		}
	},
	{
		Constants.FunctionCall,
		new ModuleRequestHandler(HandleFunctionCall)
		{
			ParametersRequired =
			{
				new ModuleRequestParameter(Constants.Function),
				new ModuleRequestParameter(Constants.Arguments)
				{
					ParameterType = JTokenType.Object,
					Optional = true
				}
			}
		}
	},
	{
		Constants.FunctionResponse,
		new ModuleRequestHandler(HandleFunctionResponse)
		{
			ParametersRequired =
			{
				new ModuleRequestParameter(Constants.RequestId),
				new ModuleRequestParameter(Constants.Data)
				{
					ParameterType = JTokenType.Object,
					Optional = false
				}
			}
		}
	},
	{
		Constants.RegisterHook,
		new ModuleRequestHandler(HandleRegisterHook)
		{
			ParametersRequired =
			{
				new ModuleRequestParameter(Constants.Hook)
			}
		}
	},
	{
		Constants.TriggerHook,
		new ModuleRequestHandler(HandleTriggerHook)
		{
			ParametersRequired =
			{
				new ModuleRequestParameter(Constants.Hook),
				new ModuleRequestParameter(Constants.Data)
				{
					Optional = true,
					ParameterType = JTokenType.Object
				}
			}
		}
	},
	{
		Constants.DeclareFunction,
		new ModuleRequestHandler(HandleDeclareFunction)
		{
			ParametersRequired =
			{
				new ModuleRequestParameter(Constants.Function)
			}
		}
	}
};

internal static void HandleRequest(Module module, string data)
{
	try
	{
		if (data == null)
			return;

		var request = JObject.Parse(data);

		var type = request[Constants.Type]?.ToObject<string>();
		var requestId = request[Constants.RequestId]?.ToObject<string>();

		if (type == null)
		{
			SendError(module, Constants.Errors.UnknownRequest);
			return;
		}
		if (requestId == null)
		{
			SendError(module, Constants.Errors.InvalidRequestId);
			return;
		}

		if (!RequestHandlers.ContainsKey(type))
		{
			SendError(module, Constants.Errors.UnknownRequest, requestId);
			return;
		}

		var moduleRequestHandler = RequestHandlers[type];
		foreach (var parameter in moduleRequestHandler.ParametersRequired)
		{
			if (request.ContainsKey(parameter.ParameterName))
			{
				if (request[parameter.ParameterName].Type != parameter.ParameterType)
				{
					SendError(module, Constants.Errors.MalformedRequest, requestId);
					return;
				}
			}
			else
			{
				if (!parameter.Optional)
				{
					SendError(module, Constants.Errors.MalformedRequest, requestId);
					return;
				}
			}
		}
		moduleRequestHandler.Handler.Invoke(module, requestId, request);
	}
	catch (JsonReaderException e)
	{
		SendError(module, Constants.Errors.MalformedRequest);
		Console.WriteLine($"Data: '{data}'. Error: {e.ToString()}");
	}
}

internal static void OnModuleDisconnected(Module module)
{
	// TODO recheck dependencies, hooks, registered modules, unregistered modules.
}

internal static void TriggerHook(Module module, string hook, bool sticky, bool force = false)
{
	// @param module is triggering a hook
	// if @param force is true, all modules gets the hook, regardless of whether they want it or not
	var hookName = module.ModuleID + "." + hook;
	foreach(var registeredModule in RegisteredModules.Values)
	{
		if(force || registeredModule.RegisteredHooks.Contains(hookName))
		{
			Send(
				registeredModule,
				new JObject
				{
					[Constants.RequestId] = DateTime.Now.Ticks,
					[Constants.Type] = Constants.HookCalled,
					[Constants.Hook] = hookName
				}
			);
		}
	}
	if(sticky)
	{
		// TODO sticky this hook somewhere so that new modules can get it
	}
}

private static void HandleModuleRegistration(Module module, string requestId, JObject request)
{
	module.ModuleID = request[Constants.ModuleId].ToObject<string>();
	module.Version = request[Constants.Version].ToObject<string>();
	module.RegistrationRequestID = requestId;
	module.Dependencies.Clear();

	var dependencies = request[Constants.Dependencies]?.ToObject<JObject>();

	if (dependencies == null || dependencies.Count == 0)
	{
		RegisteredModules.Add(module.ModuleID, module);

		module.Registered = true;

		Send(
			module,
			new JObject
			{
				[Constants.RequestId] = requestId,
				[Constants.Type] = Constants.ModuleRegistered
			}
		);
	}
	else
	{
		module.Registered = false;

		foreach(var token in dependencies)
		{
			if(token.Value.Type != JTokenType.String)
				continue;
			module.Dependencies.Add(token.Key, token.Value.ToObject<string>());
		}
		UnRegisteredModules.Add(module.ModuleID, module);
	}

	RecalculateAllModulesDependencies();
}

private static void RecalculateAllModulesDependencies()
{
	var satisfiedModules = new List<string>();
	foreach(var module in UnRegisteredModules)
	{
		// For each module, check if the dependencies are satisfied.
		var dependencySatisfied = true;
		foreach(var dependencyRequired in module.Value.Dependencies)
		{
			if(!RegisteredModules.ContainsKey(dependencyRequired.Key) && !UnRegisteredModules.ContainsKey(dependencyRequired.Key))
			{
				dependencySatisfied = false;
				break;
			}
			// TODO CHECK VERSION AS WELL
		}
		if(dependencySatisfied)
		{
			satisfiedModules.Add(module.Value.ModuleID);
		}
	}

	foreach(var moduleId in satisfiedModules)
	{
		var module = UnRegisteredModules[moduleId];
		RegisteredModules.Add(module.ModuleID, module);
		UnRegisteredModules.Remove(moduleId);

		Send(
			module,
			new JObject
			{
				[Constants.RequestId] = module.RegistrationRequestID,
				[Constants.Type] = Constants.ModuleRegistered
			}
		);
	}
}

private static void HandleFunctionCall(Module originModule, string requestId, JObject request)
{
	if (!originModule.Registered)
	{
		SendError(originModule, Constants.Errors.UnregisteredModule, requestId);
		return;
	}

	var function = request[Constants.Function].ToObject<string>();
	if (!IsFunctionName(function, out var moduleName, out var functionName))
	{
		SendError(originModule, Constants.Errors.UnknownFunction, requestId);
		return;
	}

	if (!RegisteredModules.ContainsKey(moduleName))
	{
		SendError(originModule, Constants.Errors.UnknownModule);
		return;
	}

	var recieverModule = RegisteredModules[moduleName];
	if (!recieverModule.DeclaredFunctions.Contains(functionName))
	{
		SendError(originModule, Constants.Errors.UnknownFunction);
		return;
	}

	if (RequestOrigins.ContainsKey(requestId))
	{
		if (RequestOrigins[requestId] != originModule.ModuleID)
		{
			// There's already a requestId that's supposed to return to
			// a different module. Let the module know that it's invalid
			// so that we can prevent response-hijacking.
			SendError(originModule, Constants.Errors.InvalidRequestId, requestId);
			return;
		}
	}
	else
	{
		RequestOrigins.Add(requestId, originModule.ModuleID);
	}

	if (request[Constants.Arguments] == null)
	{
		request.Add(Constants.Arguments, new JObject());
	}

	// We're all done processing the request.
	// Now just proxy the request that we got to the destination module
	request[Constants.Function].Replace(functionName);

	Send(recieverModule, request);
}

private static void HandleFunctionResponse(Module module, string requestId, JObject request)
{
	if (!module.Registered)
	{
		SendError(module, Constants.Errors.UnregisteredModule, requestId);
		return;
	}

	var data = request[Constants.Data].ToObject<JObject>();

	if (!RequestOrigins.ContainsKey(requestId))
	{
		// If the given requestId does not contain an origin,
		// drop the packet entirely
		return;
	}

	var originModuleId = RequestOrigins[requestId];

	if (!RegisteredModules.ContainsKey(originModuleId))
	{
		// The origin module has probably disconnected.
		// Drop the packet entirely
		return;
	}

	Send(RegisteredModules[originModuleId], request);
}

private static void HandleRegisterHook(Module module, string requestId, JObject request)
{
	// @param module wants to listen for a hook
	var hook = request[Constants.Hook].ToObject<string>();

	if(!module.RegisteredHooks.Contains(hook))
		module.RegisteredHooks.Add(hook);

	Send(
		module,
		new JObject
		{
			[Constants.RequestId] = requestId,
			[Constants.Type] = Constants.HookRegistered
		}
	);
}

private static void HandleTriggerHook(Module module, string requestId, JObject request)
{
	// @param module is triggering a hook
	var hook = request[Constants.Hook].ToObject<string>();
	var data = request[Constants.Data]?.ToObject<JObject>();

	TriggerHook(module, hook, false);

	Send(
		module,
		new JObject
		{
			[Constants.RequestId] = requestId,
			[Constants.Type] = Constants.HookTriggered
		}
	);
}

private static void HandleDeclareFunction(Module module, string requestId, JObject request)
{
	var function = request[Constants.Function].ToObject<string>();
	if (!module.DeclaredFunctions.Contains(function))
		module.DeclaredFunctions.Add(function);

	Send(
		module,
		new JObject
		{
			[Constants.RequestId] = requestId,
			[Constants.Type] = Constants.FunctionDeclared,
			[Constants.Function] = function
		}
	);
}

private static void Send(Module module, JObject data)
{
	var stringified = data.ToString(Constants.JsonFormatting) + "\n";
	var writeBuffer = Encoding.UTF8.GetBytes(stringified);
	module.SendBytes(writeBuffer);
}

private static void SendError(Module module, string error, string? requestId = null)
{
	if (requestId == null)
	{
		Send(
			module,
			new JObject
			{
				[Constants.Type] = Constants.Error,
				[Constants.Error] = error
			}
		);
	}
	else
	{
		Send(
			module,
			new JObject
			{
				[Constants.Type] = Constants.Error,
				[Constants.RequestId] = requestId,
				[Constants.Error] = error
			}
		);
	}
}

private static bool IsFunctionName(string name, out string moduleName, out string functionName)
{
	moduleName = string.Empty;
	functionName = string.Empty;

	if (!name.Contains("."))
		return false;

	var parts = name.Split(".", StringSplitOptions.None);
	if (parts.Length != 2)
		return false;

	foreach (var letter in parts[0])
	{
		if (!char.IsLetterOrDigit(letter) && !(letter == '-') && !(letter == '_'))
			return false;
	}

	foreach (var letter in parts[1])
	{
		if (!char.IsLetterOrDigit(letter) && !(letter == '_'))
			return false;
	}

	moduleName = parts[0];
	functionName = parts[1];

	return true;
}
*/
