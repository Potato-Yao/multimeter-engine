use crate::get_running_flag;
use crate::monitor::cross_platform::CrossPlatform;
#[cfg(feature = "fake-sensors")]
use crate::monitor::fake::Fake;
#[cfg(all(target_os = "linux", not(feature = "fake-sensors")))]
use crate::monitor::linux::Linux;
#[cfg(all(target_os = "windows", not(feature = "fake-sensors")))]
use crate::monitor::windows::Windows;
use crate::util::admin::is_admin;
use crate::util::data_container::DataContainer;
use crate::util::info_map::InfoMap;
#[cfg(any(feature = "web-api", feature = "native-api"))]
use crate::util::payload::PayLoad;
use anyhow::{Result, anyhow};
use lazy_static::lazy_static;
use log::{debug, error, warn};
use std::sync::{Arc, LazyLock, Mutex, MutexGuard, OnceLock};
use std::thread;
use std::time::Duration;
#[cfg(any(feature = "web-api", feature = "native-api"))]
use tracing::instrument;

mod cross_platform;
#[cfg(feature = "fake-sensors")]
mod fake;

mod model;
pub use self::model::Device;

#[cfg(target_os = "linux")]
mod linux;

#[cfg(windows)]
mod windows;

// #[allow(unused)]
pub trait QueryField {
    fn query(&self, key: &str, attach: Option<&InfoMap>) -> QueryResult;
}

/// the option inside Found means whether finding the corresponding field or not
#[allow(unused)]
#[derive(Debug)]
pub enum QueryResult {
    Found(Option<DataContainer>),
    NotFound,
}

#[derive(Debug)]
pub struct QueryRequest {
    pub target: String,
    pub parameter: Option<InfoMap>,
}

// #[macro_export]
// macro_rules! insert_data {
//     ($map:expr, $key:expr, $val:expr) => {
//         $map.insert(
//             $key,
//             Some(DataContainer::from($val)),
//         )
//     };
// }

trait Updater: Send + Sync {
    fn update_once(&mut self, device: &mut Device) -> Result<()>;

    fn update_slow(&mut self, device: &mut Device) -> Result<()>;

    fn update(&mut self, device: &mut Device) -> Result<()>;

    fn shutdown(&mut self) -> Result<()>;
}

static QUERY_MANAGER: OnceLock<Arc<Mutex<dyn Updater>>> = OnceLock::new();

static DEVICE: LazyLock<Arc<Mutex<Device>>> =
    LazyLock::new(|| Arc::new(Mutex::new(Device::default())));

pub fn query_device() -> Result<MutexGuard<'static, Device>> {
    DEVICE.lock().map_err(|e| anyhow!(e.to_string()))
}

#[instrument]
#[cfg(any(feature = "web-api", feature = "native-api"))]
pub fn query_info(request: QueryRequest) -> Result<PayLoad> {
    debug!("Query Request: {:?}", request);
    if QUERY_MANAGER.get().is_none() {
        init()?;
    }

    if QUERY_STATEMENTS.contains(&request.target.as_str()) {
        let device = DEVICE.lock().map_err(|e| anyhow!(e.to_string()))?;
        match QueryField::query(&*device, request.target.as_str(), None) {
            QueryResult::Found(Some(value)) => Ok(PayLoad {
                value,
                addition: None,
            }),
            QueryResult::Found(None) => Err(anyhow::anyhow!(
                "No data available for target: {}",
                request.target
            )),
            QueryResult::NotFound => {
                Err(anyhow::anyhow!("Unknown query target: {}", request.target))
            }
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

    if !is_admin() {
        warn!(
            "The engine is not running under admin permission, some operation may be restricted!"
        );
    }

    #[cfg(all(windows, not(feature = "fake-sensors")))]
    let manager = Windows::build()?;
    #[cfg(all(target_os = "linux", not(feature = "fake-sensors")))]
    let manager = Linux::build()?;
    #[cfg(feature = "fake-sensors")]
    let manager = Fake::build()?;

    let general_manager = CrossPlatform::build()?;

    let _ = QUERY_MANAGER.set(manager.clone());
    let _ = QUERY_MANAGER.set(general_manager.clone());

    {
        let mut mgr = manager.lock().map_err(|e| anyhow!(e.to_string()))?;
        let mut general_mgr = general_manager.lock().map_err(|e| anyhow!(e.to_string()))?;
        let mut device = DEVICE.lock().map_err(|e| anyhow!(e.to_string()))?;
        if let Err(e) = mgr.update_once(&mut device) {
            error!("Update failed: {}", e);
        }
        if let Err(e) = mgr.update_slow(&mut device) {
            error!("Update failed: {}", e);
        }
        if let Err(e) = mgr.update(&mut device) {
            error!("Update failed: {}", e);
        }

        if let Err(e) = general_mgr.update_once(&mut device) {
            error!("Update failed: {}", e);
        }
        if let Err(e) = general_mgr.update_slow(&mut device) {
            error!("Update failed: {}", e);
        }
        if let Err(e) = general_mgr.update(&mut device) {
            error!("Update failed: {}", e);
        }
    }

    let manager_normal = manager.clone();
    let general_manager_normal = general_manager.clone();
    thread::spawn(move || {
        while get_running_flag() {
            {
                let mut mgr = manager_normal.lock().unwrap();
                let mut general_mgr = general_manager_normal.lock().unwrap();
                let mut device = DEVICE.lock().unwrap();
                if let Err(e) = mgr.update(&mut device) {
                    error!("Update failed: {}", e);
                }
                if let Err(e) = general_mgr.update(&mut device) {
                    error!("General update failed: {}", e);
                }
            }
            thread::sleep(Duration::from_millis(500));
        }
    });

    let manager_slow = manager.clone();
    let general_manager_slow = general_manager.clone();
    thread::spawn(move || {
        while get_running_flag() {
            {
                let mut mgr = manager_slow.lock().unwrap();
                let mut general_mgr = general_manager_slow.lock().unwrap();
                let mut device = DEVICE.lock().unwrap();
                if let Err(e) = mgr.update_slow(&mut device) {
                    error!("Update failed: {}", e);
                }
                if let Err(e) = general_mgr.update_slow(&mut device) {
                    error!("Update failed: {}", e);
                }
            }
            thread::sleep(Duration::from_millis(10000));
        }
    });

    Ok(())
}

pub fn shutdown() -> Result<()> {
    if let Some(manager) = QUERY_MANAGER.get() {
        let mut mgr = manager.lock().map_err(|e| anyhow!(e.to_string()))?;
        return mgr.shutdown();
    }

    Ok(())
}

lazy_static! {
    // THE CODE BELOW IS SCRIPT GENERATED, DON'T CHANGE THEM DIRECTLY! CHANGE THE SCRIPT sensor_map.py INSTEAD
    pub static ref QUERY_STATEMENTS: Vec<&'static str> = vec![
        "bat_capacity_designed",
        "bat_capacity_max",
        "bat_capacity_remain",
        "bat_count",
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
        "disk_disk",
        "disk_disk_size",
        "disk_partition",
        "disk_temperature_first",
        "disk_temperature_last",
        "fan_rpm_cpu",
        "fan_rpm_gpu",
        "fan_rpm_mid",
        "gpu_clock_rms",
        "gpu_mem_clock_rms",
        "gpu_name",
        "gpu_power",
        "gpu_temperature",
        "gpu_usage",
        "mem_available",
        "mem_percentage",
        "mem_swap_total",
        "mem_swap_used",
        "mem_total",
        "mem_used",
        "os_activated",
        "os_host_name",
        "os_kernel_version",
        "os_name",
        "os_version",
    ];
    // THE CODE ABOVE IS SCRIPT GENERATED, DON'T CHANGE THEM DIRECTLY! CHANGE THE SCRIPT sensor_map.py INSTEAD
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::monitor::QueryField;

    #[test]
    fn test_query() {
        #[cfg(feature = "web-api")]
        {
            use chrono::Utc;
            let request = QueryRequest {
                // target: "cpu_power".to_string(),
                // target: "cpu_temperature".to_string(),
                // target: "bat_rate".to_string(),
                // target: "gpu_name".to_string(),
                // target: "gpu_temperature".to_string(),
                target: "bat_capacity_max".to_string(),
                parameter: None,
            };
            init().unwrap();
            let start = Utc::now();
            let result = query_info(request);
            let end = Utc::now();
            println!("DEVICE: {:?}", DEVICE.lock().unwrap());
            println!("Time consumed: {} ms", (end - start).num_milliseconds());
            assert!(result.is_ok());
        }
        #[cfg(not(feature = "web-api"))]
        {
            init().unwrap();
            assert!(query_device().unwrap().cpu.package_temperature.is_some());
        }
    }

    #[test]
    fn test_query_generator() {
        init().unwrap();
        println!("{:?}", DEVICE.lock().unwrap().cpu.query("cpu_name", None));
    }
}
