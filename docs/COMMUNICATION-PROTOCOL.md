# Communication protocol

These are documented in JSON format for ease of reading. As of now, juno only supports JSON formats. We can, however, expand that to support MsgPack in the future, if throughput / encode-decode speed is an issue.

## Error response

```jsonc
{
    "requestId": "module1-1234567890",
    "type": 0,
    "error": 1 // See: error-codes.md
}
```

------------

## Initialization:

### Request

```jsonc
{
    "requestId": "module1-1234567890",
    "type": 1,
    "moduleId": "module1",
    "version": "1.0.0", // Follows Semver
    "dependencies": {
        "module2": "1.0.0"
    }
}
```

### Response

```jsonc
{
    "requestId": "module1-1234567890",
    "type": 2
}
```

------------

## Function call

### Request

```jsonc
{
    "requestId": "module1-1234567890",
    "type": 3,
    "function": "module2.calculateSum",
    "data": {
        "values": [
            1,
            2,
            3,
            4,
            5
        ]
    }
}
```

### Response

```jsonc
{
    "requestId": "module1-1234567890",
    "type": 4,
    "data": 15
}
```

------------

## Hook registration

### Request

```jsonc
{
    "requestId": "module1-1234567890",
    "type": 5,
    "hook": "users.passwordChanged" // Will listen for the "passwordChanged" hook of the "users" module
}
```

### Response

```jsonc
{
    "requestId": "module1-1234567890",
    "type": 6
}
```

------------

## Trigger hook

### Request

```jsonc
{
    "requestId": "module1-1234567890",
    "type": 7,
    "hook": "passwordChanged",
    "data": {
        "userId": "testUser"
    }
}
```

### Response

```jsonc
{
    "requestId": "module1-1234567890",
    "type": 8
}
```

### Hook data sent on all listening modules

```jsonc
{
    "requestId": "unique-request-id",
    "type": 8,
    "hook": "users.passwordChanged",
    "data": {
        "userId": "testUser"
    }
}
```

------------

## Declare function

### Request

```jsonc
{
    "requestId": "module1-1234567890",
    "type": 9,
    "function": "passwordChanged"
}
```

### Response

```jsonc
{
    "requestId": "module1-1234567890",
    "type": 10
}
```
