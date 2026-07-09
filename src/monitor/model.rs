#![allow(dead_code)]

use crate::monitor::{ConditionQueryStrategy, QueryResult, query_process};
use crate::util::data_container::DataContainer;
use crate::util::info_map::InfoMap;
use multimeter_engine_macros::QueryGenerator;
use sysinfo::Process;

#[derive(Debug, Clone)]
pub enum SystemPackageManager {
    Apt,
    Dnf,
    Pacman,
}

impl From<SystemPackageManager> for DataContainer {
    fn from(value: SystemPackageManager) -> Self {
        match value {
            SystemPackageManager::Apt => DataContainer::from("apt"),
            SystemPackageManager::Dnf => DataContainer::from("dnf"),
            SystemPackageManager::Pacman => DataContainer::from("pacman"),
        }
    }
}

#[derive(Default, Debug, Clone, QueryGenerator)]
pub struct Model {
    #[query(nest)]
    pub system: System,
    #[query(nest)]
    pub battery: Battery,
    #[query(nest)]
    pub cpu: CPU,
    #[query(nest)]
    pub gpu: GPU,
    #[query(nest)]
    pub ram: RAM,
    #[query(nest)]
    pub fans: Fans,
    #[query(nest)]
    pub motherboard: Motherboard,
    #[query(nest)]
    pub disk: Disk,
    #[query(nest)]
    pub network: Network,
}

/// some info costs too much memory or time to store and update, take them as variable may not be a good choice.
/// like running processes. they costs too much memory to store, data vary rapidly and cannot update by perform partly insert and delete on origin data.
/// so instead of take a Vec to store them and reconstruct it every time, the better choice is just leave an interface for querying.
/// this is what the [VirtualDevice] used for.
#[derive(Default, Debug, Clone, QueryGenerator)]
pub struct VirtualDevice;

#[derive(Default, Debug, Clone, QueryGenerator)]
pub struct System {
    #[query(key = "os_name")]
    pub os_name: Option<String>,
    #[query(key = "os_version")]
    pub os_version: Option<String>,
    #[query(key = "os_kernel_version")]
    pub kernel_version: Option<String>,
    #[query(key = "os_host_name")]
    pub host_name: Option<String>,
    #[query(key = "os_package_manager")]
    pub package_manager: Option<SystemPackageManager>,
    #[query(key = "os_activated")]
    pub is_activated: Option<bool>,
    #[query(key = "os_process", function = "get_os_process")]
    pub process: VirtualDevice,
}

impl System {
    fn get_os_process(&self, attach: &Option<InfoMap>) -> QueryResult {
        os_process_query(attach)
    }
}

pub(crate) fn os_process_query(attach: &Option<InfoMap>) -> QueryResult {
    let strategy = process_strategy(attach);

    match query_process(strategy) {
        Ok(processes) => QueryResult::Found(Some(DataContainer::from(processes))),
        Err(_) => QueryResult::Found(None),
    }
}

fn process_strategy(attach: &Option<InfoMap>) -> ConditionQueryStrategy<Process> {
    let mut strategy = ConditionQueryStrategy::default();
    let Some(attach) = attach else {
        return strategy;
    };

    if let Some(limit) = attach.get("limit").and_then(data_container_to_usize) {
        strategy.limit = Some(limit);
    }

    let descending = attach
        .get("descending")
        .and_then(data_container_to_bool)
        .unwrap_or(false);

    match attach.get("sort_by").and_then(data_container_to_str) {
        Some("pid") => {
            strategy.comparer = if descending {
                |a, b| b.pid().as_u32().cmp(&a.pid().as_u32())
            } else {
                |a, b| a.pid().as_u32().cmp(&b.pid().as_u32())
            };
        }
        Some("name") => {
            strategy.comparer = if descending {
                |a, b| b.name().cmp(a.name())
            } else {
                |a, b| a.name().cmp(b.name())
            };
        }
        Some("memory") => {
            strategy.comparer = if descending {
                |a, b| b.memory().cmp(&a.memory())
            } else {
                |a, b| a.memory().cmp(&b.memory())
            };
        }
        Some("virtual_memory") => {
            strategy.comparer = if descending {
                |a, b| b.virtual_memory().cmp(&a.virtual_memory())
            } else {
                |a, b| a.virtual_memory().cmp(&b.virtual_memory())
            };
        }
        Some("cpu") | Some("cpu_usage") | None => {
            strategy.comparer = if descending {
                |a, b| b.cpu_usage().total_cmp(&a.cpu_usage())
            } else {
                |a, b| a.cpu_usage().total_cmp(&b.cpu_usage())
            };
        }
        Some(_) => {}
    }

    strategy
}

fn data_container_to_usize(value: &DataContainer) -> Option<usize> {
    match value {
        DataContainer::Int(value) => (*value).try_into().ok(),
        DataContainer::UnsignedLong(value) => (*value).try_into().ok(),
        _ => None,
    }
}

fn data_container_to_bool(value: &DataContainer) -> Option<bool> {
    match value {
        DataContainer::Boolean(value) => Some(*value),
        _ => None,
    }
}

fn data_container_to_str(value: &DataContainer) -> Option<&str> {
    match value {
        DataContainer::Text(value) => Some(value.as_str()),
        _ => None,
    }
}

#[derive(Default, Debug, Clone, QueryGenerator)]
pub struct Battery {
    #[query(key = "bat_capacity_designed")]
    pub designed_capacity: Option<f64>, // the maximum capacity the battery should have
    #[query(key = "bat_capacity_max")]
    pub actually_capacity: Option<f64>, // the maximum capacity the battery actually has
    #[query(key = "bat_capacity_remain")]
    pub remain_capacity: Option<f64>, // the current remaining capacity
    #[query(key = "bat_voltage")]
    pub voltage: Option<f64>, // the voltage battery supplies
    #[query(key = "bat_current")]
    pub current: Option<f64>, // the current battery supplies
    #[query(key = "bat_rate")]
    pub rate: Option<f64>, // charge or discharge power
    #[query(key = "bat_state")]
    pub is_charging: Option<bool>, // charging state
}

impl Battery {
    pub fn get_health_percentage(&self) -> Option<f64> {
        let designed_capacity = self.designed_capacity?;
        let actually_capacity = self.actually_capacity?;
        if designed_capacity == 0.0 {
            return None;
        }
        Some(actually_capacity / designed_capacity)
    }

    pub fn get_remain_percentage(&self) -> Option<f64> {
        let remain_capacity = self.remain_capacity?;
        let actually_capacity = self.actually_capacity?;
        if actually_capacity == 0.0 {
            return None;
        }
        Some((remain_capacity / actually_capacity) * 100.0)
    }
}

#[allow(clippy::upper_case_acronyms)]
#[derive(Default, Debug, Clone, QueryGenerator)]
pub struct CPU {
    #[query(key = "cpu_name")]
    pub name: Option<String>, // name of the CPU
    #[query(key = "cpu_usage")]
    pub usage: Option<f64>, // cpu usage
    #[query(key = "cpu_temperature")]
    pub package_temperature: Option<f64>, // package temperature
    #[query(key = "cpu_temperature_avg")]
    pub average_temperature: Option<f64>, // average core temperature
    #[query(key = "cpu_power")]
    pub power: Option<f64>, // power consumption
    #[query(key = "cpu_clock_begin_index")]
    pub clock_begin_index: Option<i32>, // sensor index start
    #[query(key = "cpu_clock_end_index")]
    pub clock_end_index: Option<i32>, // sensor index end
    #[query(key = "cpu_clock_avg")]
    pub clock: Option<f64>, // calculated clock speed
    #[query(key = "cpu_load")]
    pub load: Option<f64>, // cpu load
    #[query(key = "cpu_voltage")]
    pub voltage: Option<f64>, // cpu voltage
}

#[allow(clippy::upper_case_acronyms)]
#[derive(Default, Debug, Clone, QueryGenerator)]
pub struct GPU {
    #[query(key = "gpu_name")]
    pub name: Option<String>,
    #[query(key = "gpu_power_usage")]
    pub power_usage: Option<f64>,
    #[query(key = "gpu_temperature")]
    pub temperature: Option<f64>,
    #[query(key = "gpu_clock")]
    pub clock: Option<i32>,
    #[query(key = "gpu_max_temperature")]
    pub max_temperature: Option<f64>,
    #[query(key = "gpu_power")]
    pub power: Option<f64>,
    #[query(key = "gpu_clock_rms")]
    pub speed: Option<f64>,
    #[query(key = "gpu_mem_clock_rms")]
    pub mem_clock: Option<i32>,
    #[query(key = "gpu_mem_total")]
    pub mem_total: Option<f64>,
    #[query(key = "gpu_mem_free")]
    pub mem_free: Option<f64>,
    #[query(key = "gpu_mem_used")]
    pub mem_used: Option<f64>,
    #[query(key = "gpu_mem_usage")]
    pub mem_usage: Option<f64>,
}

#[allow(clippy::upper_case_acronyms)]
#[derive(Default, Debug, Clone, QueryGenerator)]
pub struct RAM {
    // the reason used, free and total all here, even total can be calculated by the other two is some api gives a total, which is more accuracy than the sum of free and used
    #[query(key = "mem_used")]
    pub used_size: Option<f64>,
    #[query(key = "mem_available")]
    pub free_size: Option<f64>,
    #[query(key = "mem_total")]
    pub total_size: Option<f64>,
    #[query(key = "mem_swap_total")]
    pub total_swap: Option<f64>,
    #[query(key = "mem_swap_used")]
    pub used_swap: Option<f64>,
    #[query(key = "mem_swap_free")]
    pub free_swap: Option<f64>,
}

impl RAM {
    pub fn get_memory_used_percentage(&self) -> Option<f64> {
        let used_size = self.used_size?;
        let total_size = self.total_size?;
        if total_size == 0.0 {
            return None;
        }
        Some((used_size / total_size) * 100.0)
    }
}

#[derive(Default, Debug, Clone, QueryGenerator)]
pub struct Fans {
    #[query(key = "fan_rpm_cpu")]
    pub cpu_speed: Option<i32>,
    #[query(key = "fan_rpm_gpu")]
    pub gpu_speed: Option<i32>,
    #[query(key = "fan_rpm_mid")]
    pub mid_speed: Option<i32>,
}

#[derive(Default, Debug, Clone, QueryGenerator)]
pub struct Motherboard {
    #[query(key = "motherboard_name")]
    pub name: Option<String>,
}

#[derive(Default, Debug, Clone, QueryGenerator)]
pub struct Disk {
    #[query(key = "disk_disk")]
    disk_list: Option<Vec<String>>,
}

#[derive(Default, Debug, Clone, QueryGenerator)]
pub struct Network;

#[derive(Default, Debug, Clone)]
pub struct VirtualProcess {}
