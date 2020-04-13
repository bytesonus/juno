use async_std::io::Result;

#[cfg(target_family = "unix")]
mod socket_server_unix;
#[cfg(target_family = "windows")]
mod socket_server_windows;

#[cfg(target_family = "windows")]
pub async fn listen(socket_path: &str) -> Result<()> {
	socket_server_windows::listen(socket_path).await
}

#[cfg(target_family = "unix")]
pub async fn listen(socket_path: &str) -> Result<()> {
	socket_server_unix::listen(socket_path).await
}

#[cfg(target_family = "windows")]
pub async fn on_exit() {
	socket_server_windows::on_exit().await;
}

#[cfg(target_family = "unix")]
pub async fn on_exit() {
	socket_server_unix::on_exit().await;
}
