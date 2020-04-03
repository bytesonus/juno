# Concepts

## TL;DR

- Modules connect.
- Modules initialize with `moduleId`, `version` and `dependencies`.
- Modules declare functions
- Modules can call functions declared by other modules
- Modules can listen for hooks (kinda like events)
- Modules can trigger hooks on all listening modules

## The actual explaination

Each module is something that connects with Gotham either through the Unix Socket or through the Inet Socket.

Once a module connects, it needs to register itself with gotham. This is done by sending a command to initialize itself with a `moduleId`, `version` and `dependencies`.

Once gotham has recieved the registration command, and there are no issues, it will respond with a successful registration command, otherwise it will respond with an error code. The list of gotham error codes can be found [here](./ERROR-CODES.md).

Gotham will then check if dependencies are satisfied. If it is, it will send a `gotham.activated` hook.

Then, if one of the dependencies of your module disconnects, gotham will recognize that all your dependencies are not satisfied and will send a `gotham.deactivated` hook.

Every request sent to gotham MUST be sent with a `requestId` and `type` key.  
This request ID will be the identifier for which request a particular response is meant for. This `requestId` can be any unique string. However, to avoid collisions, it is recommended to use `<module-id>-<unix-timestamp>` as a format for the `requestId`.  
The `type` is a number which mentions the type of request / response being sent.

You can then declare functions to gotham, which can then be called by other modules.

For example: if a module called `module1` declares a function called `calculateSum`, another module (`module2`) can call that function by calling the function `module1.calculateSum`. However, when `module1` recieves the function-call request, the function name will be stripped down to just `calculateSum`.

The parameters passed can be any key-value pair (kinda like a JSON object). It's upto the calling module (in the above example, `module2`) to ensure that the right parameters are passed to the functions. It's also the responsibility of the called module (`module1`, in the above example) to ensure that the parameters are validated before executing the function.

When a module is responding to a function call, it can choose to respond with a successful function response, (along with the response data, if any), or it can choose to respond with an error (with an error code).

A module can also choose to trigger a hook. A hook is like an event. Any module can trigger a hook along with some data and other modules can choose to listen for the hook and respond to actions accordingly.

A module can also choose to listen for a hook. The module will only recieve a hook if it is listening for a hook.

However, some hooks are forced onto the module. For example, hooks from Gotham, such as `gotham.activated` and `gotham.deactivated` are always forced onto modules.

You can find the protocol of communication [here](./COMMUNICATION-PROTOCOL.md).
