use crate::constants;

use async_std::io::Result;

mod socket_server_inet;
#[cfg(target_family = "unix")]
mod socket_server_unix;

#[cfg(target_family = "windows")]
pub async fn listen(socket_path: &str) -> Result<()> {
	if crate::get_connection_type() == constants::connection_types::UNIX_SOCKET {
		panic!("Unix sockets are not supported in windows. How did you even get here?");
	} else if crate::get_connection_type() == constants::connection_types::INET_SOCKET {
		socket_server_inet::listen(socket_path).await
	} else {
		panic!("Any other connection type other than INet sockets and Unix Sockets are not implemented yet");
	}
}

#[cfg(target_family = "unix")]
pub async fn listen(socket_path: &str) -> Result<()> {
	if crate::get_connection_type() == constants::connection_types::UNIX_SOCKET {
		socket_server_unix::listen(socket_path).await
	} else if crate::get_connection_type() == constants::connection_types::INET_SOCKET {
		socket_server_inet::listen(socket_path).await
	} else {
		panic!("Any other connection type other than INet sockets and Unix Sockets are not implemented yet");
	}
}

#[cfg(target_family = "windows")]
pub async fn on_exit() {
	if crate::get_connection_type() == constants::connection_types::UNIX_SOCKET {
		panic!("Unix sockets are not supported in windows. How did you even get here?");
	} else if crate::get_connection_type() == constants::connection_types::INET_SOCKET {
		socket_server_inet::on_exit().await;
	} else {
		panic!("Any other connection type other than INet sockets and Unix Sockets are not implemented yet");
	}
}

#[cfg(target_family = "unix")]
pub async fn on_exit() {
	if crate::get_connection_type() == constants::connection_types::UNIX_SOCKET {
		socket_server_unix::on_exit().await;
	} else if crate::get_connection_type() == constants::connection_types::INET_SOCKET {
		socket_server_inet::on_exit().await;
	} else {
		panic!("Any other connection type other than INet sockets and Unix Sockets are not implemented yet");
	}
}
