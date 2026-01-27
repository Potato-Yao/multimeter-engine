use crate::monitor::windows::Windows;
use crate::util::data_container::DataContainer;
use crate::util::payload::PayLoad;
use anyhow::{Result, anyhow};
use lazy_static::lazy_static;
use log::error;
use std::collections::HashMap;
use std::sync::{Arc, LazyLock, Mutex, OnceLock};
use std::thread;
use std::time::Duration;

mod hardware_model;
mod windows;

pub type InfoMap = HashMap<String, DataContainer>;

pub struct QueryRequest {
    pub target: String,
    pub parameter: Option<InfoMap>,
}

trait Updater: Send + Sync {
    fn update(&mut self, map: &mut HashMap<&str, Option<DataContainer>>) -> Result<()>;
}

static QUERY_MANAGER: OnceLock<Arc<Mutex<dyn Updater>>> = OnceLock::new();
static INFO_MAP: LazyLock<Mutex<HashMap<&str, Option<DataContainer>>>> = LazyLock::new(|| {
    let mut m: HashMap<&str, Option<DataContainer>> = HashMap::new();
    QUERY_STATEMENTS.iter().for_each(|e| {
        m.insert(e, None);
    });

    Mutex::new(m)
});

pub fn query_info(request: QueryRequest) -> Result<PayLoad> {
    if QUERY_MANAGER.get().is_none() {
        init()?;
    }

    if QUERY_STATEMENTS.contains(&request.target.as_str()) {
        // add special handling here

        let map = INFO_MAP.lock().map_err(|e| anyhow!(e.to_string()))?;
        println!("INFO: {:?}", map);
        if let Some(value) = map.get(request.target.as_str()) {
            if let Some(data) = value {
                Ok(PayLoad {
                    value: data.clone(),
                    addition: None,
                })
            } else {
                Err(anyhow::anyhow!(
                    "No data available for target: {}",
                    request.target
                ))
            }
        } else {
            Err(anyhow::anyhow!(
                "Target not found in info map: {}",
                request.target
            ))
        }
    } else {
        Err(anyhow::anyhow!("Unknown query target: {}", request.target))
    }
}

fn init() -> Result<()> {
    #[cfg(windows)]
    let manager = Windows::build()?;

    let _ = QUERY_MANAGER.set(manager.clone());

    let mgr = manager.clone();
    let mut mgr = mgr.lock().map_err(|e| anyhow!(e.to_string()))?;
    let mut map = &mut INFO_MAP.lock().map_err(|e| anyhow!(e.to_string()))?;
    if let Err(e) = mgr.update(&mut map) {
        error!("Update failed: {}", e);
    }

    thread::spawn(move || {
        loop {
            {
                let mut mgr = manager.lock().unwrap();
                let mut map = &mut INFO_MAP.lock().unwrap();
                if let Err(e) = mgr.update(&mut map) {
                    error!("Update failed: {}", e);
                    break;
                }
            }
            thread::sleep(Duration::from_millis(500));
        }
    });

    Ok(())
}

lazy_static! {
    pub static ref QUERY_STATEMENTS: Vec<&'static str> = vec![
        "cpu_name",
        "cpu_load_total",
        "cpu_temperature",
        "cpu_power",
        "cpu_voltage",
        "cpu_clock_avg",
        "cpu_clock_rms",
        "cpu_clock_max",
        "cpu_usage",
        "gpu_name",
        "gpu_temperature",
        "gpu_power",
        "gpu_voltage",
        "gpu_clock_rms",
        "mem_total",
        "mem_available",
        "bat_capacity_max",
        "bat_capacity_remain",
        "bat_capacity_designed",
        "bat_voltage",
        "bat_rate",
        "bat_state",
        "os_activated",
        "disk_partition",
        "disk_disk",
        "disk_partition_detail",
        "disk_disk_detail",
    ];
    pub static ref INTERNAL_QUERY_STATEMENTS: Vec<&'static str> = {
        let mut v = QUERY_STATEMENTS.clone();
        #[cfg(windows)]
        {
            v.push("clock_begin_index");
            v.push("clock_end_index");
        }

        v
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query() {
        let request = QueryRequest {
            target: "cpu_voltage".to_string(),
            parameter: None,
        };
        let result = query_info(request);
        println!("{:?}", result);
    }
}
