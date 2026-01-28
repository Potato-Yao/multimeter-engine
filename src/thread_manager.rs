use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};
use std::thread::JoinHandle;
use anyhow::Result;

static THREADS: LazyLock<Mutex<HashMap<String, JoinHandle<()>>>> = LazyLock::new(|| {
    Mutex::new(HashMap::new())
});

pub fn register_thread(name: &str, handle: JoinHandle<()>) -> Result<()> {
    todo!();
    if let Ok(mut threads) = THREADS.lock() {
        threads.insert(name.to_string(), handle);
    } else {
        return Err(anyhow::anyhow!("Failed to acquire thread registry lock"));
    }

    Ok(())
}
