use anyhow::Result;
use config::Config;
use log::debug;
#[cfg(feature = "web-api")]
use log::{error, info};
#[cfg(feature = "web-api")]
use std::ffi::{CStr, CString, c_char};
#[cfg(feature = "web-api")]
use std::future::Future;
use std::sync::Mutex;
#[cfg(feature = "web-api")]
use std::sync::OnceLock;
#[cfg(feature = "web-api")]
use tokio::runtime::Runtime;
pub mod cli;
pub mod config;
pub mod external_program;
pub mod monitor;
pub mod thread_manager;
pub mod util;

#[cfg(any(feature = "web-api", feature = "native-api"))]
pub mod web;

#[cfg(feature = "stress-test")]
pub mod stress_test;
pub mod system_migration;

static KEEP_RUNNING: Mutex<bool> = Mutex::new(true);

#[cfg(feature = "web-api")]
static RUNTIME: OnceLock<Runtime> = OnceLock::new();

pub fn get_running_flag() -> bool {
    *KEEP_RUNNING.lock().unwrap()
}

#[cfg(feature = "web-api")]
pub fn runtime() -> &'static Runtime {
    RUNTIME.get_or_init(|| Runtime::new().expect("failed to create Tokio runtime"))
}

#[cfg(feature = "web-api")]
fn spawn_server_task<F>(future: F)
where
    F: Future + Send + 'static,
    F::Output: Send + 'static,
{
    if let Ok(handle) = tokio::runtime::Handle::try_current() {
        handle.spawn(future);
    } else {
        runtime().spawn(future);
    }
}

#[cfg(any(feature = "web-api", feature = "native-api"))]
pub fn process_command(input: String) -> String {
    match web::handle_request(input) {
        Ok(response) | Err(response) => serde_json::to_string(&response).unwrap(),
    }
}

pub fn engine_init(config: Config) -> Result<()> {
    if config.sensor.enable {
        monitor::init()?;
    }

    #[cfg(feature = "web-api")]
    {
        if config.tcp.enable {
            let port = config.tcp.port;
            spawn_server_task(async move {
                if let Err(e) = web::server::start_tcp_server(port).await {
                    error!("TCP server error on port {}: {}", port, e);
                }
            });
            info!("TCP server started on port {}", port);
        }

        if config.http.enable {
            info!("HTTP server is not implemented yet");
        }
    }

    if config.tui.enable {
        debug!("TUI is not implemented yet");
    }

    Ok(())
}

#[unsafe(no_mangle)]
#[cfg(feature = "web-api")]
pub extern "C" fn multimeter_init() -> i32 {
    let config = Config::load().unwrap_or_default();
    match engine_init(config) {
        Ok(_) => 0,
        Err(_) => -1,
    }
}

/// # Safety
/// safe
#[unsafe(no_mangle)]
#[cfg(feature = "web-api")]
pub unsafe extern "C" fn multimeter_query(input: *const c_char) -> *mut c_char {
    if input.is_null() {
        return std::ptr::null_mut();
    }

    let c_str = unsafe { CStr::from_ptr(input) };
    let input_str = c_str.to_string_lossy().into_owned();

    let output_str = process_command(input_str);

    CString::new(output_str).unwrap().into_raw()
}

#[unsafe(no_mangle)]
#[cfg(feature = "web-api")]
pub extern "C" fn multimeter_shutdown() {
    let _ = monitor::shutdown();
    if let Ok(mut flag) = KEEP_RUNNING.lock() {
        *flag = false;
    }
}

pub fn shutdown() -> Result<()> {
    debug!("Shutting down");

    // fixme shutdown doesnt work
    // let payload = PayLoad {
    //     value: "Shutdown initiated.".into(),
    //     addition: None,
    // };
    //
    // if let Err(e) = monitor::shutdown() {
    //     return Err(anyhow!(e.to_string()));
    // }
    //
    // *KEEP_RUNNING.lock().unwrap() = false;

    Ok(())
}
