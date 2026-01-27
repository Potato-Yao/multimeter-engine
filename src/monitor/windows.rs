use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use crate::external_program::lhm_helper::LhmHelper;
use crate::monitor::{Updater, INTERNAL_QUERY_STATEMENTS};
use crate::util::admin::is_admin;
use crate::util::data_container::DataContainer;
use serde::Deserialize;

#[cfg(windows)]
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Sensor {
    id: String,
    name: String,
    index: i32,
    #[serde(rename = "Type")]
    kind: String,
    info: String,
}

#[cfg(windows)]
pub struct Windows {
    index_map: HashMap<String, i32>,
    lhm_helper: LhmHelper,
}

impl Updater for Windows {
    fn update(&mut self, map: &mut HashMap<&str, Option<DataContainer>>) -> Result<()> {
        self.lhm_helper.update()?;
        for (k, v) in map.iter_mut() {
            if let Some(index) = self.index_map.get(*k) && *index != -1 {
                let value = self.query_sensor_value(*index).map_err(|e| anyhow!(e))?;
                *v = Some(DataContainer::from(value));
            } else {
                *v = None;
            }
        }

        Ok(())
    }
}

#[cfg(windows)]
impl Windows {
    pub fn build() -> Result<Arc<Mutex<Self>>> {
        if !is_admin() {
            return Err(anyhow::anyhow!("Windows is not admin"));
        }

        let mut lhm_helper = LhmHelper::connect()?;
        let lhm_sensors_json = lhm_helper.query_hardware()?;
        let sensors: Vec<Sensor> = serde_json::from_str(&lhm_sensors_json)?;
        let index_map = Self::build_index_map(&sensors);

        let manager = Windows {
            index_map,
            lhm_helper,
        };

        let manager = Arc::new(Mutex::new(manager));

        Ok(manager)
    }

    fn build_index_map(sensors: &Vec<Sensor>) -> HashMap<String, i32> {
        let mut map = HashMap::new();
        INTERNAL_QUERY_STATEMENTS.iter().for_each(|e| {
            map.insert(e.to_string(), -1);
        });

        for sensor in sensors {
            // THE CODE BELOW IS SCRIPT GENERATED, DON'T CHANGE THEM DIRECTLY! CHANGE THE SCRIPT .\sensor_map.py INSTEAD
            if sensor.name == "CPU Package" && sensor.info == "Temperature" {
                map.insert("cpu_temperature".to_string(), sensor.index);
            } else if sensor.name == "CPU Package" && sensor.info == "Power" {
                map.insert("cpu_power".to_string(), sensor.index);
            } else if sensor.name == "CPU Core" && sensor.info == "Voltage" {
                map.insert("cpu_voltage".to_string(), sensor.index);
            } else if sensor.name == "GPU Core" && sensor.info == "Temperature" {
                map.insert("gpu_temperature".to_string(), sensor.index);
            } else if sensor.name == "GPU Package" && sensor.info == "Power" {
                map.insert("gpu_power".to_string(), sensor.index);
            } else if sensor.name == "Memory Available" && sensor.info == "Data" {
                map.insert("mem_available".to_string(), sensor.index);
            } else if sensor.name == "Fully-Charged Capacity" && sensor.info == "Energy" {
                map.insert("bat_capacity_max".to_string(), sensor.index);
            } else if sensor.name == "Remaining Capacity" && sensor.info == "Energy" {
                map.insert("bat_capacity_remain".to_string(), sensor.index);
            } else if sensor.name == "Designed Capacity" && sensor.info == "Energy" {
                map.insert("bat_capacity_designed".to_string(), sensor.index);
            } else if sensor.name == "Voltage" && sensor.info == "Voltage" {
                map.insert("bat_voltage".to_string(), sensor.index);
            } else if sensor.name == "Charge Rate" && sensor.info == "Power" {
                map.insert("bat_rate".to_string(), sensor.index);
            } else if sensor.name == "Discharge Rate" && sensor.info == "Power" {
                map.insert("bat_rate".to_string(), sensor.index);
            } else if sensor.name == "Charge/Discharge Rate" && sensor.info == "Power" {
                map.insert("bat_rate".to_string(), sensor.index);
            }
            // THE CODE ABOVE IS SCRIPT GENERATED, DON'T CHANGE THEM DIRECTLY! CHANGE THE SCRIPT .\sensor_map.py INSTEAD
        }

        map
    }

    fn query_sensor_value(&mut self, index: i32) -> Result<f64> {
        let value = self.lhm_helper.get_value(index)?;

        Ok(value)
    }
}
