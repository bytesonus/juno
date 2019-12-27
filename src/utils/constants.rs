extern crate clap;

use clap::{crate_authors, crate_name, crate_version};

pub const APP_NAME: &str = crate_name!();
pub const APP_VERSION: &str = crate_version!();
pub const APP_AUTHORS: &str = crate_authors!();

pub const DEFAULT_SOCKET_LOCATION: &str = "../gotham.sock";
pub const DEFAULT_MODULES_LOCATION: &str = "../modules/";

#[allow(dead_code)]
pub mod request_types {
	pub const ERROR: u32 = 0;

	pub const MODULE_REGISTRATION: u32 = 1;
	pub const MODULE_REGISTERED: u32 = 2;

	pub const FUNCTION_CALL: u32 = 3;
	pub const FUNCTION_RESPONSE: u32 = 4;

	pub const REGISTER_HOOK: u32 = 5;
	pub const HOOK_REGISTERED: u32 = 6;

	pub const TRIGGER_HOOK: u32 = 7;
	pub const HOOK_TRIGGERED: u32 = 8;

	pub const DECLARE_FUNCTION: u32 = 9;
	pub const FUNCTION_DECLARED: u32 = 10;
}

#[allow(dead_code)]
pub mod errors {
	pub const MALFORMED_REQUEST: u32 = 0;

	pub const INVALID_REQUEST_ID: u32 = 1;
	pub const UNKNOWN_REQUEST: u32 = 2;
	pub const UNREGISTERED_MODULE: u32 = 3;
	pub const UNKNOWN_MODULE: u32 = 4;
	pub const UNKNOWN_FUNCTION: u32 = 5;
}
