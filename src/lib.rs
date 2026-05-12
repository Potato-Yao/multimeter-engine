use anyhow::{Result};
use log::debug;
#[cfg(feature = "web-api")]
use std::ffi::{CStr, CString, c_char};

use std::sync::Mutex;
pub mod external_program;
pub mod monitor;
pub mod thread_manager;
pub mod util;

#[cfg(any(feature = "web-api", feature = "native-api"))]
pub mod web;

#[cfg(feature = "stress-test")]
pub mod stress_test;

static KEEP_RUNNING: Mutex<bool> = Mutex::new(true);

pub fn get_running_flag() -> bool {
    *KEEP_RUNNING.lock().unwrap()
}

#[cfg(any(feature = "web-api", feature = "native-api"))]
pub fn process_command(input: String) -> String {
    match web::handle_request(input) {
        Ok(response) | Err(response) => serde_json::to_string(&response).unwrap(),
    }
}

pub fn engine_init() -> Result<()> {
    monitor::init()?;

    Ok(())
}

#[unsafe(no_mangle)]
#[cfg(feature = "web-api")]
pub extern "C" fn multimeter_init() -> i32 {
    match engine_init() {
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
