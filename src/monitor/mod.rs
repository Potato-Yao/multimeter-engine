use crate::monitor::windows::Windows;
use crate::util::data_container::DataContainer;
use crate::util::payload::PayLoad;
use anyhow::{anyhow, Result};
use lazy_static::lazy_static;
use log::{debug, error};
use std::collections::HashMap;
use std::sync::{Arc, LazyLock, Mutex, OnceLock};
use std::thread;
use std::time::Duration;

mod hardware_model;
mod windows;

pub type InfoMap = HashMap<String, DataContainer>;

#[derive(Debug)]
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
    debug!("Query Request: {:?}", request);
    if QUERY_MANAGER.get().is_none() {
        init()?;
    }

    if QUERY_STATEMENTS.contains(&request.target.as_str()) {
        // add special handling here

        let map = INFO_MAP.lock().map_err(|e| anyhow!(e.to_string()))?;
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
    debug!("Initializing Query Manager");
    #[cfg(windows)]
    let manager = Windows::build()?;

    let _ = QUERY_MANAGER.set(manager.clone());

    {
        let mut mgr = manager.lock().map_err(|e| anyhow!(e.to_string()))?;
        let mut map = INFO_MAP.lock().map_err(|e| anyhow!(e.to_string()))?;
        if let Err(e) = mgr.update(&mut map) {
            error!("Update failed (initial): {}", e);
        }
    }
    thread::spawn(move || {
        loop {
            {
                let mut mgr = manager.lock().unwrap();
                let mut map = INFO_MAP.lock().unwrap();
                if let Err(e) = mgr.update(&mut map) {
                    error!("Update failed: {}", e);
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
        for _ in 0..30 {
            let request = QueryRequest {
                target: "cpu_power".to_string(),
                parameter: None,
            };
            let result = query_info(request);
            assert!(result.is_ok());
            thread::sleep(Duration::from_millis(200));
        }
    }
}
