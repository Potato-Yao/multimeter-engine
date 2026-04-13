pub mod external_program;
pub mod monitor;
pub mod thread_manager;
pub mod util;
pub mod web;

use crate::util::payload::PayLoad;
use anyhow::{Result, anyhow};
use futures::{SinkExt, StreamExt};
use log::debug;
use std::ffi::{CStr, CString, c_char};
use std::sync::Mutex;
use tokio::net::TcpStream;
use tokio_util::codec::{Framed, LinesCodec};

static KEEP_RUNNING: Mutex<bool> = Mutex::new(true);

pub fn get_running_flag() -> bool {
    *KEEP_RUNNING.lock().unwrap()
}

pub fn process_command(input: String) -> String {
    match web::handle_request(input) {
        Ok(response) | Err(response) => serde_json::to_string(&response).unwrap(),
    }
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

#[unsafe(no_mangle)]
pub extern "C" fn multimeter_init() -> i32 {
    match monitor::init() {
        Ok(_) => 0,
        Err(_) => -1,
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn multimeter_query(input: *const c_char) -> *mut c_char {
    if input.is_null() {
        return std::ptr::null_mut();
    }

    let c_str = unsafe { CStr::from_ptr(input) };
    let input_str = c_str.to_string_lossy().into_owned();

    let output_str = process_command(input_str);

    CString::new(output_str).unwrap().into_raw()
}

#[unsafe(no_mangle)]
pub extern "C" fn multimeter_shutdown() {
    let _ = monitor::shutdown();
    if let Ok(mut flag) = KEEP_RUNNING.lock() {
        *flag = false;
    }
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
