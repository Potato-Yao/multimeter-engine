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
use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt;
use std::ops::{Deref, DerefMut};
use std::sync::{Arc, Mutex, MutexGuard, OnceLock};
use std::thread;
use std::time::Duration;
use sysinfo::Process;
#[cfg(any(feature = "web-api", feature = "native-api"))]
use tracing::instrument;

mod cross_platform;
#[cfg(feature = "fake-sensors")]
mod fake;

mod model;
pub use self::model::Model;

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

#[derive(Debug, Clone)]
pub struct ProcessSnapshot {
    pub pid: u32,
    pub parent: Option<u32>,
    pub name: String,
    pub cmd: Vec<String>,
    pub exe: Option<String>,
    pub cwd: Option<String>,
    pub root: Option<String>,
    pub environ: Vec<String>,
    pub status: String,
    pub start_time: u64,
    pub run_time: u64,
    pub cpu_usage: f64,
    pub memory: u64,
    pub virtual_memory: u64,
    pub total_read_bytes: u64,
    pub read_bytes: u64,
    pub total_written_bytes: u64,
    pub written_bytes: u64,
}

impl ProcessSnapshot {
    fn from_process(process: &Process) -> Self {
        let disk_usage = process.disk_usage();

        Self {
            pid: process.pid().as_u32(),
            parent: process.parent().map(|pid| pid.as_u32()),
            name: process.name().to_string_lossy().into_owned(),
            cmd: process
                .cmd()
                .iter()
                .map(|value| value.to_string_lossy().into_owned())
                .collect(),
            exe: process
                .exe()
                .map(|path| path.to_string_lossy().into_owned()),
            cwd: process
                .cwd()
                .map(|path| path.to_string_lossy().into_owned()),
            root: process
                .root()
                .map(|path| path.to_string_lossy().into_owned()),
            environ: process
                .environ()
                .iter()
                .map(|value| value.to_string_lossy().into_owned())
                .collect(),
            status: format!("{:?}", process.status()),
            start_time: process.start_time(),
            run_time: process.run_time(),
            cpu_usage: process.cpu_usage() as f64,
            memory: process.memory(),
            virtual_memory: process.virtual_memory(),
            total_read_bytes: disk_usage.total_read_bytes,
            read_bytes: disk_usage.read_bytes,
            total_written_bytes: disk_usage.total_written_bytes,
            written_bytes: disk_usage.written_bytes,
        }
    }
}

impl From<ProcessSnapshot> for DataContainer {
    fn from(value: ProcessSnapshot) -> Self {
        let mut data = HashMap::new();

        data.insert("pid".to_string(), DataContainer::from(value.pid as u64));
        data.insert(
            "parent".to_string(),
            value
                .parent
                .map(|pid| DataContainer::from(pid as u64))
                .unwrap_or(DataContainer::Null),
        );
        data.insert("name".to_string(), DataContainer::from(value.name));
        data.insert("cmd".to_string(), DataContainer::from(value.cmd));
        data.insert(
            "exe".to_string(),
            value
                .exe
                .map(DataContainer::from)
                .unwrap_or(DataContainer::Null),
        );
        data.insert(
            "cwd".to_string(),
            value
                .cwd
                .map(DataContainer::from)
                .unwrap_or(DataContainer::Null),
        );
        data.insert(
            "root".to_string(),
            value
                .root
                .map(DataContainer::from)
                .unwrap_or(DataContainer::Null),
        );
        data.insert("environ".to_string(), DataContainer::from(value.environ));
        data.insert("status".to_string(), DataContainer::from(value.status));
        data.insert(
            "start_time".to_string(),
            DataContainer::from(value.start_time),
        );
        data.insert("run_time".to_string(), DataContainer::from(value.run_time));
        data.insert(
            "cpu_usage".to_string(),
            DataContainer::from(value.cpu_usage),
        );
        data.insert("memory".to_string(), DataContainer::from(value.memory));
        data.insert(
            "virtual_memory".to_string(),
            DataContainer::from(value.virtual_memory),
        );
        data.insert(
            "total_read_bytes".to_string(),
            DataContainer::from(value.total_read_bytes),
        );
        data.insert(
            "read_bytes".to_string(),
            DataContainer::from(value.read_bytes),
        );
        data.insert(
            "total_written_bytes".to_string(),
            DataContainer::from(value.total_written_bytes),
        );
        data.insert(
            "written_bytes".to_string(),
            DataContainer::from(value.written_bytes),
        );

        DataContainer::Object(data)
    }
}

impl From<Vec<ProcessSnapshot>> for DataContainer {
    fn from(value: Vec<ProcessSnapshot>) -> Self {
        DataContainer::Array(value.into_iter().map(DataContainer::from).collect())
    }
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
    fn update_once(&mut self, model: &mut Model) -> Result<()>;

    fn update_slow(&mut self, model: &mut Model) -> Result<()>;

    fn update(&mut self, model: &mut Model) -> Result<()>;

    fn shutdown(&mut self) -> Result<()>;
}

pub struct Device {
    model: Model,
    platform_updater: Arc<Mutex<dyn Updater>>,
    cross_platform_updater: Arc<Mutex<CrossPlatform>>,
}

impl Device {
    fn new(
        platform_updater: Arc<Mutex<dyn Updater>>,
        cross_platform_updater: Arc<Mutex<CrossPlatform>>,
    ) -> Self {
        Self {
            model: Model::default(),
            platform_updater,
            cross_platform_updater,
        }
    }

    fn update_once(&mut self) {
        let Self {
            model,
            platform_updater,
            cross_platform_updater,
        } = self;

        handle_update(
            platform_updater,
            model,
            "Update failed",
            |updater, model| {
                updater.update_once(model)?;
                updater.update_slow(model)?;
                updater.update(model)
            },
        );
        handle_update(
            cross_platform_updater,
            model,
            "Update failed",
            |updater, model| {
                updater.update_once(model)?;
                updater.update_slow(model)?;
                updater.update(model)
            },
        );
    }

    fn update(&mut self) {
        let Self {
            model,
            platform_updater,
            cross_platform_updater,
        } = self;

        handle_update(
            platform_updater,
            model,
            "Update failed",
            |updater, model| updater.update(model),
        );
        handle_update(
            cross_platform_updater,
            model,
            "General update failed",
            |updater, model| updater.update(model),
        );
    }

    fn update_slow(&mut self) {
        let Self {
            model,
            platform_updater,
            cross_platform_updater,
        } = self;

        handle_update(
            platform_updater,
            model,
            "Update failed",
            |updater, model| updater.update_slow(model),
        );
        handle_update(
            cross_platform_updater,
            model,
            "Update failed",
            |updater, model| updater.update_slow(model),
        );
    }

    fn shutdown(&mut self) -> Result<()> {
        self.platform_updater
            .lock()
            .map_err(|e| anyhow!(e.to_string()))?
            .shutdown()?;
        self.cross_platform_updater
            .lock()
            .map_err(|e| anyhow!(e.to_string()))?
            .shutdown()
    }
}

fn handle_update<U: Updater + ?Sized>(
    updater: &Arc<Mutex<U>>,
    model: &mut Model,
    error_message: &str,
    update: impl FnOnce(&mut U, &mut Model) -> Result<()>,
) {
    match updater.lock() {
        Ok(mut updater) => {
            if let Err(e) = update(&mut *updater, model) {
                error!("{}: {}", error_message, e);
            }
        }
        Err(e) => error!("{}: {}", error_message, e),
    }
}

impl Deref for Device {
    type Target = Model;

    fn deref(&self) -> &Self::Target {
        &self.model
    }
}

impl DerefMut for Device {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.model
    }
}

impl fmt::Debug for Device {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Device")
            .field("model", &self.model)
            .finish_non_exhaustive()
    }
}

static DEVICE: OnceLock<Arc<Mutex<Device>>> = OnceLock::new();

pub fn query_device() -> Result<MutexGuard<'static, Device>> {
    if DEVICE.get().is_none() {
        init()?;
    }

    DEVICE
        .get()
        .ok_or_else(|| anyhow!("Device is not initialized"))?
        .lock()
        .map_err(|e| anyhow!(e.to_string()))
}

#[instrument]
#[cfg(any(feature = "web-api", feature = "native-api"))]
pub fn query_info(request: QueryRequest) -> Result<PayLoad> {
    debug!("Query Request: {:?}", request);
    if DEVICE.get().is_none() {
        init()?;
    }

    if request.target == "process" {
        return match crate::monitor::model::process_query(request.parameter.as_ref()) {
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
        };
    }

    if QUERY_STATEMENTS.contains(&request.target.as_str()) {
        let device = query_device()?;
        match QueryField::query(&device.model, request.target.as_str(), None) {
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
    if DEVICE.get().is_some() {
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

    let mut device = Device::new(manager.clone(), general_manager.clone());
    device.update_once();

    if DEVICE.set(Arc::new(Mutex::new(device))).is_err() {
        return Ok(());
    }

    let device_normal = DEVICE.get().unwrap().clone();
    thread::spawn(move || {
        while get_running_flag() {
            {
                let mut device = device_normal.lock().unwrap();
                device.update();
            }
            thread::sleep(Duration::from_millis(500));
        }
    });

    let device_slow = DEVICE.get().unwrap().clone();
    thread::spawn(move || {
        while get_running_flag() {
            {
                let mut device = device_slow.lock().unwrap();
                device.update_slow();
            }
            thread::sleep(Duration::from_millis(10000));
        }
    });

    Ok(())
}

pub fn shutdown() -> Result<()> {
    if let Some(device) = DEVICE.get() {
        return device
            .lock()
            .map_err(|e| anyhow!(e.to_string()))?
            .shutdown();
    }

    Ok(())
}

pub struct ConditionQueryStrategy<T> {
    pub comparer: fn(&T, &T) -> Ordering,
    // true for keeping the element
    pub keep: fn(&T) -> bool,
    // keep first n elements
    pub limit: Option<usize>,
}

impl Default for ConditionQueryStrategy<Process> {
    fn default() -> Self {
        Self {
            comparer: |a, b| a.cpu_usage().total_cmp(&b.cpu_usage()),
            keep: |_| true,
            limit: None,
        }
    }
}

pub fn query_process(strategy: ConditionQueryStrategy<Process>) -> Result<Vec<ProcessSnapshot>> {
    let device = query_device()?;
    let cross_platform_updater = device
        .cross_platform_updater
        .lock()
        .map_err(|e| anyhow!(e.to_string()))?;
    let source = cross_platform_updater.get_process();
    let mut result: Vec<&Process> = source
        .iter()
        .copied()
        .filter(|e| (strategy.keep)(*e))
        .collect();
    result.sort_by(|a, b| (strategy.comparer)(*a, *b));

    if let Some(limit) = strategy.limit {
        result.truncate(limit);
    }

    Ok(result
        .into_iter()
        .map(ProcessSnapshot::from_process)
        .collect())
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
        "process",
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
            // println!("DEVICE: {:?}", DEVICE.get());
            println!("Time consumed: {} ms", (end - start).num_milliseconds());
            assert!(result.is_ok());
        }
        #[cfg(not(feature = "web-api"))]
        {
            init().unwrap();
            assert!(query_device().is_ok());
        }
    }

    #[test]
    fn test_query_generator() {
        init().unwrap();
        println!(
            "{:?}",
            DEVICE
                .get()
                .unwrap()
                .lock()
                .unwrap()
                .cpu
                .query("cpu_name", None)
        );
    }
}
