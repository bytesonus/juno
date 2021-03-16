use crate::utils::ConnectionType;

use tokio::io::Result;

mod socket_server_inet;
#[cfg(target_family = "unix")]
mod socket_server_unix;

#[cfg(target_family = "windows")]
pub async fn listen(
	connection_type: ConnectionType,
	socket_path: &str,
) -> Result<()> {
	match connection_type {
		ConnectionType::UnixSocket => {
			panic!("Unix sockets are not supported in windows. How did you even get here?")
		}
		ConnectionType::InetSocket => {
			socket_server_inet::listen(socket_path).await
		}
	}
}

#[cfg(target_family = "unix")]
pub async fn listen(
	connection_type: ConnectionType,
	socket_path: &str,
) -> Result<()> {
	match connection_type {
		ConnectionType::UnixSocket => {
			socket_server_unix::listen(socket_path).await
		}
		ConnectionType::InetSocket => {
			socket_server_inet::listen(socket_path).await
		}
	}
}

#[cfg(target_family = "windows")]
pub async fn on_exit(connection_type: ConnectionType) {
	match connection_type {
		ConnectionType::UnixSocket => {
			panic!("Unix sockets are not supported in windows. How did you even get here?")
		}
		ConnectionType::InetSocket => socket_server_inet::on_exit().await,
	}
}

#[cfg(target_family = "unix")]
pub async fn on_exit(connection_type: ConnectionType) {
	match connection_type {
		ConnectionType::UnixSocket => socket_server_unix::on_exit().await,
		ConnectionType::InetSocket => socket_server_inet::on_exit().await,
	}
}
