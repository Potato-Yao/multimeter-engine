pub mod external_program;
pub mod monitor;
pub mod util;
pub mod web;
pub mod thread_manager;

use crate::util::payload::PayLoad;
use anyhow::{Result, anyhow};
use futures::{SinkExt, StreamExt};
use log::debug;
use std::sync::Mutex;
use tokio::net::TcpStream;
use tokio_util::codec::{Framed, LinesCodec};

static KEEP_RUNNING: Mutex<bool> = Mutex::new(true);

pub fn get_running_flag() -> bool {
    *KEEP_RUNNING.lock().unwrap()
}

pub fn handle_request(socket: TcpStream) {
    tokio::spawn(async move {
        let mut framed = Framed::new(socket, LinesCodec::new());

        while let Some(Ok(line)) = framed.next().await {
            match web::handle_request(line) {
                Ok(response) | Err(response) => {
                    let response_str = serde_json::to_string(&response).unwrap();
                    let _ = framed.send(response_str).await;
                }
            }
        }
    });
}

pub fn shutdown() -> Result<PayLoad> {
    todo!();
    debug!("Shutting down");

    let payload = PayLoad {
        value: "Shutdown initiated.".into(),
        addition: None,
    };

    if let Err(e) = monitor::shutdown() {
        return Err(anyhow!(e.to_string()));
    }

    *KEEP_RUNNING.lock().unwrap() = false;

    Ok(payload)
}
