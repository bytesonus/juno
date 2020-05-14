use clap::{crate_authors, crate_name, crate_version};

pub const APP_NAME: &str = crate_name!();
pub const APP_VERSION: &str = crate_version!();
pub const APP_AUTHORS: &str = crate_authors!();

pub const DEFAULT_SOCKET_LOCATION: &str = "../juno.sock";

pub mod request_keys {
	pub const TYPE: &str = "type";
	pub const REQUEST_ID: &str = "requestId";
	pub const MODULE_ID: &str = "moduleId";
	pub const VERSION: &str = "version";
	pub const DEPENDENCIES: &str = "dependencies";
	pub const ERROR: &str = "error";
	pub const FUNCTION: &str = "function";
	pub const HOOK: &str = "hook";
	pub const ARGUMENTS: &str = "arguments";
	pub const DATA: &str = "data";
}

pub mod juno_hooks {
	pub const ACTIVATED: &str = "activated";
	pub const DEACTIVATED: &str = "deactivated";
	pub const MODULE_ACTIVATED: &str = "moduleActivated";
	pub const MODULE_DEACTIVATED: &str = "moduleDeactivated";
}

pub mod request_types {
	pub const ERROR: u64 = 0;

	pub const REGISTER_MODULE_REQUEST: u64 = 1;
	pub const REGISTER_MODULE_RESPONSE: u64 = 2;

	pub const FUNCTION_CALL_REQUEST: u64 = 3;
	pub const FUNCTION_CALL_RESPONSE: u64 = 4;

	pub const REGISTER_HOOK_REQUEST: u64 = 5;
	pub const REGISTER_HOOK_RESPONSE: u64 = 6;

	pub const TRIGGER_HOOK_REQUEST: u64 = 7;
	pub const TRIGGER_HOOK_RESPONSE: u64 = 8;

	pub const DECLARE_FUNCTION_REQUEST: u64 = 9;
	pub const DECLARE_FUNCTION_RESPONSE: u64 = 10;
}

pub mod errors {
	pub const MALFORMED_REQUEST: u32 = 0;

	pub const INVALID_REQUEST_ID: u32 = 1;
	pub const UNKNOWN_REQUEST: u32 = 2;
	pub const UNREGISTERED_MODULE: u32 = 3;
	pub const UNKNOWN_MODULE: u32 = 4;
	pub const UNKNOWN_FUNCTION: u32 = 5;
	pub const INVALID_MODULE_ID: u32 = 6;
	pub const DUPLICATE_MODULE: u32 = 7;
}

pub mod connection_types {
	pub const UNIX_SOCKET: u8 = 1;
	pub const INET_SOCKET: u8 = 2;
}
