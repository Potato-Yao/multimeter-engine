use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};
use std::thread::JoinHandle;
use anyhow::Result;

static _THREADS: LazyLock<Mutex<HashMap<String, JoinHandle<()>>>> = LazyLock::new(|| {
    Mutex::new(HashMap::new())
});

pub fn register_thread(_name: &str, _handle: JoinHandle<()>) -> Result<()> {
    todo!();
    // if let ok(mut threads) = threads.lock() {
    //     threads.insert(name.to_string(), handle);
    // } else {
    //     return err(anyhow::anyhow!("failed to acquire thread registry lock"));
    // }
    //
    // ok(())
}
