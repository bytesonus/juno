extern crate clap;

use clap::{crate_authors, crate_name, crate_version};

pub const APP_NAME: &str = crate_name!();
pub const APP_VERSION: &str = crate_version!();
pub const APP_AUTHORS: &str = crate_authors!();

pub const DEFAULT_SOCKET_LOCATION: &str = "../gotham.sock";

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

pub mod gotham_hooks {
	pub const ACTIVATED: &str = "activated";
	pub const DEACTIVATED: &str = "deactivated";
}

pub mod request_types {
	pub const ERROR: u64 = 0;

	pub const MODULE_REGISTRATION: u64 = 1;
	pub const MODULE_REGISTERED: u64 = 2;

	pub const FUNCTION_CALL: u64 = 3;
	pub const FUNCTION_RESPONSE: u64 = 4;

	pub const REGISTER_HOOK: u64 = 5;
	pub const HOOK_REGISTERED: u64 = 6;

	pub const TRIGGER_HOOK: u64 = 7;
	pub const HOOK_TRIGGERED: u64 = 8;

	pub const DECLARE_FUNCTION: u64 = 9;
	pub const FUNCTION_DECLARED: u64 = 10;
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

pub mod communication_types {
	pub const UNIX_SOCKET: u8 = 1;
	pub const INET_SOCKET: u8 = 2;
}
