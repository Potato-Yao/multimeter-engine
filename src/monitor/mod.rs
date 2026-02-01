use crate::get_running_flag;
use crate::monitor::windows::Windows;
use crate::util::data_container::DataContainer;
use crate::util::payload::PayLoad;
use anyhow::{Result, anyhow};
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
    fn update_once(&mut self, map: &mut HashMap<&str, Option<DataContainer>>) -> Result<()>;

    fn update_slow(&mut self, map: &mut HashMap<&str, Option<DataContainer>>) -> Result<()>;

    fn update(&mut self, map: &mut HashMap<&str, Option<DataContainer>>) -> Result<()>;

    fn shutdown(&mut self) -> Result<()>;
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

    // debug!("Info map: {:?}", INFO_MAP.lock().map_err(|e| anyhow!(e.to_string()))?);

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

pub fn init() -> Result<()> {
    if QUERY_MANAGER.get().is_some() {
        return Ok(());
    }

    debug!("Initializing Query Manager");
    #[cfg(windows)]
    let manager = Windows::build()?;

    let _ = QUERY_MANAGER.set(manager.clone());

    {
        let mut mgr = manager.lock().map_err(|e| anyhow!(e.to_string()))?;
        let mut map = INFO_MAP.lock().map_err(|e| anyhow!(e.to_string()))?;
        if let Err(e) = mgr.update_once(&mut map) {
            error!("Update failed: {}", e);
        }
        if let Err(e) = mgr.update_slow(&mut map) {
            error!("Update failed: {}", e);
        }
        if let Err(e) = mgr.update(&mut map) {
            error!("Update failed: {}", e);
        }
    }

    let manager_normal = manager.clone();
    thread::spawn(move || {
        while get_running_flag() {
            {
                let mut mgr = manager_normal.lock().unwrap();
                let mut map = INFO_MAP.lock().unwrap();
                if let Err(e) = mgr.update(&mut map) {
                    error!("Update failed: {}", e);
                }
            }
            thread::sleep(Duration::from_millis(500));
        }
    });

    let manager_slow = manager.clone();
    thread::spawn(move || {
        while get_running_flag() {
            {
                let mut mgr = manager_slow.lock().unwrap();
                let mut map = INFO_MAP.lock().unwrap();
                if let Err(e) = mgr.update_slow(&mut map) {
                    error!("Update failed: {}", e);
                }
            }
            thread::sleep(Duration::from_millis(10000));
        }
    });

    Ok(())
}

pub fn shutdown() -> Result<()> {
    #[cfg(windows)]
    {
        if let Some(manager) = QUERY_MANAGER.get() {
            let mut mgr = manager.lock().map_err(|e| anyhow!(e.to_string()))?;
            return mgr.shutdown();
        }
    }

    Ok(())
}

lazy_static! {
    // THE CODE BELOW IS SCRIPT GENERATED, DON'T CHANGE THEM DIRECTLY! CHANGE THE SCRIPT sensor_map.py INSTEAD
    pub static ref QUERY_STATEMENTS: Vec<&'static str> = vec![
        "bat_capacity_designed",
        "bat_capacity_max",
        "bat_capacity_remain",
        "bat_rate",
        "bat_state",
        "bat_voltage",
        "cpu_clock_avg",
        "cpu_clock_first",
        "cpu_clock_last",
        "cpu_clock_max",
        "cpu_clock_rms",
        "cpu_name",
        "cpu_power",
        "cpu_temperature",
        "cpu_temperature_first",
        "cpu_temperature_last",
        "cpu_tjmax_first",
        "cpu_tjmax_last",
        "cpu_usage",
        "cpu_usage_first",
        "cpu_usage_last",
        "cpu_voltage",
        "cpu_voltage_first",
        "cpu_voltage_last",
        "disk_disk_size",
        "disk_partition",
        "disk_partition_detail",
        "disk_temperature_first",
        "disk_temperature_last",
        "gpu_clock_rms",
        "gpu_mem_clock_rms",
        "gpu_name",
        "gpu_power",
        "gpu_temperature",
        "gpu_usage",
        "mem_available",
        "mem_percentage",
        "mem_used",
        "os_activated",
    ];
    // THE CODE ABOVE IS SCRIPT GENERATED, DON'T CHANGE THEM DIRECTLY! CHANGE THE SCRIPT sensor_map.py INSTEAD
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_query() {
        let request = QueryRequest {
            target: "cpu_power".to_string(),
            parameter: None,
        };
        let result = query_info(request);
        assert!(result.is_ok());
        println!("INFO MAP: {:?}", INFO_MAP.lock().unwrap());
    }
}
