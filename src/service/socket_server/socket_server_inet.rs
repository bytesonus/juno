use std::net::SocketAddr;

use crate::{models::ModuleComm, service::data_handler};

use lazy_static::lazy_static;
use tokio::{
	io::{AsyncWriteExt, Result},
	net::{TcpListener, TcpStream},
	sync::Mutex,
	task,
};

use futures::{
	channel::mpsc::{unbounded, UnboundedSender},
	future::{self, Either},
	StreamExt,
};
use futures_util::sink::SinkExt;

lazy_static! {
	static ref CLOSE_LISTENER: Mutex<Option<UnboundedSender<()>>> =
		Mutex::new(None);
}

pub async fn listen(socket_port: &str) -> Result<()> {
	let (sender, mut receiver) = unbounded::<()>();
	CLOSE_LISTENER.lock().await.replace(sender);
	let mut close_future = receiver.next();

	let socket_server = TcpListener::bind(socket_port).await?;

	// // Setup juno module
	// let (read_data_sender, read_data_receiver) = unbounded::<Buffer>();
	// let (write_data_sender, write_data_receiver) = unbounded::<Buffer>();
	// task::spawn(async move {
	// 	let (sender, mut receiver) = unbounded::<String>();
	// 	let module_comm = ModuleComm::new_internal_comm(0, read_data_sender, sender);

	// 	let read_future = module_comm.internal_read_loop(write_data_receiver);
	// 	let write_future = module_comm.write_data_loop(&mut receiver);

	// 	future::join(read_future, write_future).await;
	// 	logger::verbose("Disconnecting internal modules...");
	// });
	// let module = juno_module::setup_juno_module(read_data_receiver, write_data_sender).await;

	log::trace!(
		"Listening for socket connections on port {}...",
		socket_port
	);
	loop {
		let accept_future = socket_server.accept();
		tokio::pin!(accept_future);
		let select_result: _ =
			future::select(accept_future, close_future).await;
		match select_result {
			Either::Left((stream, next_close_future)) => {
				close_future = next_close_future;
				log::info!("Socket connected");
				task::spawn(handle_inet_socket_client(stream));
			}
			_ => break,
		}
	}

	//drop(module);

	log::trace!("Socket server is closed.");

	Ok(())
}

pub async fn on_exit() {
	let close_sender = CLOSE_LISTENER.lock().await.take();
	if let Some(mut sender) = close_sender {
		sender.send(()).await.unwrap();
	}
}

async fn handle_inet_socket_client(stream: Result<(TcpStream, SocketAddr)>) {
	if stream.is_err() {
		log::error!("Error occured while opening socket");
		return;
	}

	let (stream, _) = stream.unwrap();
	let (write_sender, mut write_receiver) = unbounded::<String>();
	log::trace!("New MPSC channel created");

	let uuid = data_handler::new_connection_id().await;
	let (socket_reader, mut socket_writer) = stream.into_split();

	log::info!("New connection assigned ID {}", uuid);
	let module_comm = ModuleComm::new(uuid, write_sender);

	log::trace!("Polling connection ID {}", uuid);
	let task = task::spawn(async move {
		while let Some(data) = write_receiver.next().await {
			if let Err(err) = socket_writer.write_all(data.as_bytes()).await {
				log::error!("Error while writing to socket: {}", err);
			}
		}
	});
	let read_future = module_comm.read_data_loop(socket_reader);

	let _ = future::join(read_future, task).await;
	log::info!("Connection with ID {} disconnected", uuid);
}
