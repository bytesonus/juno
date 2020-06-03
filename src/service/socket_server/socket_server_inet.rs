use crate::{
	models::{juno_module, ModuleComm},
	service::data_handler,
	utils::logger,
};
use juno::connection::Buffer;

use async_std::{
	io::Result,
	net::{TcpListener, TcpStream},
	prelude::*,
	sync::Mutex,
	task,
};

use futures::{
	channel::mpsc::{unbounded, UnboundedSender},
	future::{self, Either},
};
use futures_util::sink::SinkExt;

lazy_static! {
	static ref CLOSE_LISTENER: Mutex<Option<UnboundedSender<()>>> = Mutex::new(None);
}

pub async fn listen(socket_port: &str) -> Result<()> {
	let (sender, mut receiver) = unbounded::<()>();
	CLOSE_LISTENER.lock().await.replace(sender);
	let mut close_future = receiver.next();

	let socket_server = TcpListener::bind(socket_port).await?;
	let mut incoming = socket_server.incoming();

	// Setup juno module
	let (read_data_sender, read_data_receiver) = unbounded::<Buffer>();
	let (write_data_sender, write_data_receiver) = unbounded::<Buffer>();
	task::spawn(async move {
		let (sender, mut receiver) = unbounded::<String>();
		let module_comm = ModuleComm::new_internal_comm(0, read_data_sender, sender);

		let read_future = module_comm.internal_read_loop(write_data_receiver);
		let write_future = module_comm.write_data_loop(&mut receiver);

		future::join(read_future, write_future).await;
		logger::verbose("Disconnecting internal modules...");
	});
	let module = juno_module::setup_juno_module(read_data_receiver, write_data_sender).await;

	logger::verbose(&format!(
		"Listening for socket connections on port {}...",
		socket_port
	));
	while let Either::Left((Some(stream), next_close_future)) =
		future::select(incoming.next(), close_future).await
	{
		close_future = next_close_future;
		logger::info("Socket connected");
		task::spawn(handle_inet_socket_client(stream));
	}

	drop(module);

	logger::verbose("Socket server is closed.");

	Ok(())
}

pub async fn on_exit() {
	let close_sender = CLOSE_LISTENER.lock().await.take();
	if let Some(mut sender) = close_sender {
		sender.send(()).await.unwrap();
	}
}

async fn handle_inet_socket_client(stream: Result<TcpStream>) {
	if stream.is_err() {
		logger::error("Error occured while opening socket");
		return;
	}

	let stream = stream.unwrap();
	let (sender, mut receiver) = unbounded::<String>();
	logger::verbose("New MPSC channel created");

	let uuid = data_handler::new_connection_id().await;
	logger::info(&format!("New connection assigned ID {}", uuid));
	let module_comm = ModuleComm::new_inet_comm(uuid, stream, sender);

	logger::verbose(&format!("Polling connection ID {}", uuid));
	let read_future = module_comm.read_data_loop();
	let write_future = module_comm.write_data_loop(&mut receiver);

	future::join(read_future, write_future).await;
	logger::info(&format!("Connection with ID {} disconnected", uuid));
}
