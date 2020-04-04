use crate::{
	models::ModuleComm,
	service::data_handler,
	utils::{constants, logger},
};

use async_std::{
	fs::remove_file,
	io::Result,
	net::{TcpListener, TcpStream},
	os::unix::net::{UnixListener, UnixStream},
	path::Path,
	prelude::*,
	sync::Mutex,
	task,
};

use futures::{
	channel::mpsc::{unbounded, UnboundedSender},
	future::{self, Either},
};
use futures_util::sink::SinkExt;

use rand::{thread_rng, Rng};

lazy_static! {
	static ref CLOSE_LISTENER: Mutex<Option<UnboundedSender<()>>> = Mutex::new(None);
}

pub async fn listen(socket_path: &str) -> Result<()> {
	if crate::get_connection_type() == constants::connection_types::UNIX_SOCKET {
		listen_unix_socket(socket_path).await
	} else if crate::get_connection_type() == constants::connection_types::INET_SOCKET {
		listen_inet_socket(socket_path).await
	} else {
		panic!("Any other connection type other than INet sockets and Unix Sockets are not implemented yet");
	}
}

async fn listen_unix_socket(socket_path: &str) -> Result<()> {
	let socket_path = Path::new(socket_path);
	// File lock is aquired. If the unix socket exists, then it's clearly a dangling socket. Feel free to delete it
	if socket_path.exists().await {
		logger::verbose("Removing existing unix socket");
		remove_file(socket_path).await?;
	}

	let (sender, mut receiver) = unbounded::<()>();
	CLOSE_LISTENER.lock().await.replace(sender);
	let mut close_future = receiver.next();

	let socket_server = UnixListener::bind(socket_path).await?;
	let mut incoming = socket_server.incoming();

	logger::verbose("Listening for socket connections...");
	while let Either::Left((Some(stream), next_close_future)) =
		future::select(incoming.next(), close_future).await
	{
		close_future = next_close_future;
		logger::info("Socket connected");
		task::spawn(async {
			handle_unix_socket_client(stream).await;
		});
	}

	logger::verbose("Socket server is closed.");

	Ok(())
}

async fn listen_inet_socket(socket_port: &str) -> Result<()> {
	let (sender, mut receiver) = unbounded::<()>();
	CLOSE_LISTENER.lock().await.replace(sender);
	let mut close_future = receiver.next();

	let socket_server = TcpListener::bind(socket_port).await?;
	let mut incoming = socket_server.incoming();

	logger::verbose(&format!(
		"Listening for socket connections on port {}...",
		socket_port
	));
	while let Either::Left((Some(stream), next_close_future)) =
		future::select(incoming.next(), close_future).await
	{
		close_future = next_close_future;
		logger::info("Socket connected");
		task::spawn(async {
			handle_inet_socket_client(stream).await;
		});
	}

	logger::verbose("Socket server is closed.");

	Ok(())
}

pub async fn on_exit() {
	let close_sender = CLOSE_LISTENER.lock().await.take();
	if let Some(mut sender) = close_sender {
		sender.send(()).await.unwrap();
	}
}

async fn handle_unix_socket_client(stream: Result<UnixStream>) {
	if stream.is_err() {
		logger::error("Error occured while opening socket");
		return;
	}

	let stream = stream.unwrap();
	let (sender, mut receiver) = unbounded::<String>();
	logger::verbose("New MPSC channel created");

	let mut uuid: u128 = thread_rng().gen();
	while uuid == 0 || data_handler::is_uuid_exists(&uuid).await {
		uuid = thread_rng().gen();
	}
	logger::info(&format!("New connection assigned ID {}", uuid));
	let module_comm = ModuleComm::new_unix_comm(uuid, stream, sender);

	logger::verbose(&format!("Polling connection ID {}", uuid));
	let read_future = module_comm.read_data_loop();
	let write_future = module_comm.write_data_loop(&mut receiver);

	future::join(read_future, write_future).await;
	logger::info(&format!("Connection with ID {} disconnected", uuid));
}

async fn handle_inet_socket_client(stream: Result<TcpStream>) {
	if stream.is_err() {
		logger::error("Error occured while opening socket");
		return;
	}

	let stream = stream.unwrap();
	let (sender, mut receiver) = unbounded::<String>();
	logger::verbose("New MPSC channel created");

	let mut uuid: u128 = thread_rng().gen();
	while uuid == 0 || data_handler::is_uuid_exists(&uuid).await {
		uuid = thread_rng().gen();
	}
	logger::info(&format!("New connection assigned ID {}", uuid));
	let module_comm = ModuleComm::new_inet_comm(uuid, stream, sender);

	logger::verbose(&format!("Polling connection ID {}", uuid));
	let read_future = module_comm.read_data_loop();
	let write_future = module_comm.write_data_loop(&mut receiver);

	future::join(read_future, write_future).await;
	logger::info(&format!("Connection with ID {} disconnected", uuid));
}
