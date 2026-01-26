use crate::monitor::windows::Windows;
use crate::util::data_container::DataContainer;
use crate::util::payload::PayLoad;
use anyhow::{Result, anyhow};
use lazy_static::lazy_static;
use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};

mod hardware_model;
mod windows;

pub type InfoMap = HashMap<String, DataContainer>;

pub struct QueryRequest {
    pub target: String,
    pub parameter: Option<InfoMap>,
}

trait Queryable: Send + Sync {
    fn query(&self, request: &QueryRequest) -> Result<PayLoad>;
}

static query_manager: OnceLock<Arc<Mutex<dyn Queryable>>> = OnceLock::new();

pub fn query_info(request: QueryRequest) -> Result<PayLoad> {
    if query_manager.get().is_none() {
        init()?;
    }

    if QUERY_STATEMENTS.contains(&request.target.as_str()) {
        match query_manager.get().unwrap().lock() {
            Ok(x) => x,
            Err(e) => return Err(anyhow!(e.to_string())),
        }
        .query(&request)
    } else {
        Err(anyhow::anyhow!("Unknown query target: {}", request.target))
    }
}

fn init() -> Result<()> {
    #[cfg(windows)]
    {
        let win = Windows::build()?;
        let _ = query_manager.set(win);
    }

    Ok(())
}

lazy_static! {
    pub static ref QUERY_STATEMENTS: Vec<&'static str> = vec![
        "cpu_name",
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
}
