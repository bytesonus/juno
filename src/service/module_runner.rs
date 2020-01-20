use crate::utils::logger;

use async_std::{fs::read_dir, io::Result, prelude::StreamExt, sync::Mutex};

use futures::channel::mpsc::UnboundedSender;
use futures_util::sink::SinkExt;

lazy_static! {
	static ref CLOSE_LISTENER: Mutex<Option<UnboundedSender<()>>> = Mutex::new(None);
}

pub async fn listen(modules_path: &str) -> Result<()> {
	let mut folders = read_dir(modules_path).await?;

	while let Some(folder) = folders.next().await {
		if let Err(err) = folder {
			logger::error(&format!("Unable to read folder in modules. Error: {}", err));
			continue;
		}

		let folder = folder.unwrap().path();
		let folder = folder.as_path();

		if folder.is_file().await {
			logger::verbose(&format!(
				"File {} is a file. Ignoring...",
				folder.to_str().unwrap()
			));
			continue;
		}
	}

	Ok(())
}

pub async fn on_exit() {
	let close_sender = CLOSE_LISTENER.lock().await.take();
	if let Some(mut sender) = close_sender {
		sender.send(()).await.unwrap();
	}
}
