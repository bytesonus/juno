/*
	Registration:
	{
		"requestId": "123456789",
		"type": "moduleRegistration",
		"moduleId": "module-name",
		"version": "1.0.0",
		"dependencies": {
			
		}
	}
	{"requestId": "123456789","type": "moduleRegistration","moduleId": "module-name","version": "1.0.0","dependencies": {}}

	{
		"success": true,
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
		"success": true,
		"type": "registerHook",
		"requestId": "123456789"
	}

	Declare function:
	{
		"requestId": "123456789",
		"type": "declareFunction",
		"function": "function_name"
	}
	{"requestId": "123456789", "type": "declareFunction", "function": "functionName"}

 */