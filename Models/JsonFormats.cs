/*
	Registration:
	{
		"requestId": "123456789",
		"type": "moduleRegistration",
		"moduleId": "module-name",
		"version": "1.0.0",
		"dependencies": {
			"database-module": "npm-like-versioning? idk yet"
		}
	}
	{
		"requestId": "123456789",
		"type": "moduleRegistered"
	}

	Function calls:
	{
		"requestId": "123456789",
		"type": "functionCall",
		"function": "module-name.function_name",
		"arguments": {
			"name": "value",
			"name2": "value"
		}
	}
	{
		"requestId": "123456789",
		"type": "functionResponse",
		"data": {
			"key1": "value",
			"key2": "value"
		}
	}

	Listen for hook:
	{
		"requestId": "123456789",
		"type": "registerHook",
		"hook": "module-name.hook_name"
	}
	{
		"requestId": "123456789",
		"type": "hookRegistered"
	}

	Trigger hook:
	{
		"requestId": "123456789",
		"type": "triggerHook",
		"hook": "hook_name"
	}
	- To the module that's triggering the hook:
	{
		"requestId": "123456789",
		"type": "hookTriggered",
		"hook": "hook_name"
	}
	- To the module that's listening for the hook:
	{
		"requestId": "123456789",
		"type": "hookTriggered",
		"hook": "module-name.hook_name"
	}

	Declare function:
	{
		"requestId": "123456789",
		"type": "declareFunction",
		"function": "function_name"
	}
	{
		"requestId": "123456789",
		"type": "functionDeclared",
		"function": "function_name"
	}

 */