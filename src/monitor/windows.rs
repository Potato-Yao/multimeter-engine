use crate::external_program::lhm_helper::LhmHelper;
use crate::external_program::program::{ExternalProgram, ProgramKind};
use crate::monitor::{QUERY_STATEMENTS, Updater};
use crate::util::admin::is_admin;
use crate::util::data_container::DataContainer;
use anyhow::{Result, anyhow};
use log::debug;
use regex::Regex;
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

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
    payload: WindowsPayload,
}

#[derive(Default)]
struct WindowsPayload {
    prev_bat_capacity: f64,
}

impl Updater for Windows {
    fn update_once(&mut self, map: &mut HashMap<&str, Option<DataContainer>>) -> Result<()> {
        self.set_activation_state(map);

        Ok(())
    }

    fn update_slow(&mut self, map: &mut HashMap<&str, Option<DataContainer>>) -> Result<()> {
        self.set_disk_info(map);

        Ok(())
    }

    fn update(&mut self, map: &mut HashMap<&str, Option<DataContainer>>) -> Result<()> {
        self.lhm_helper.update()?;
        for (k, v) in map.iter_mut() {
            if let Some(index) = self.index_map.get(*k)
                && *index != -1
            {
                let value = self.query_sensor_value(*index).map_err(|e| anyhow!(e))?;
                *v = Some(DataContainer::from(value));
            }
        }

        self.set_battery_state(map);
        self.set_cpu_clock(map)?;

        Ok(())
    }

    fn shutdown(&mut self) -> Result<()> {
        if let Err(e) = self.lhm_helper.disconnect() {
            return Err(anyhow!(e));
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
            payload: WindowsPayload::default(),
        };

        let manager = Arc::new(Mutex::new(manager));

        Ok(manager)
    }

    fn build_index_map(sensors: &Vec<Sensor>) -> HashMap<String, i32> {
        let mut map = HashMap::new();
        QUERY_STATEMENTS.iter().for_each(|e| {
            map.insert(e.to_string(), -1);
        });

        // THE CODE BELOW IS SCRIPT GENERATED, DON'T CHANGE THEM DIRECTLY! CHANGE THE SCRIPT sensor_map.py INSTEAD
        let regex_cpu_temperature_last = regex::Regex::new(r"^CPU Core #\d{1,2}$").unwrap();
        let regex_cpu_tjmax_last = regex::Regex::new(r"^CPU Core #\d{1,2} Distance to TjMax").unwrap();
        let regex_cpu_voltage_last = regex::Regex::new(r"^CPU Core #\d{1,2}$").unwrap();
        let regex_cpu_clock_last = regex::Regex::new(r"^CPU Core #\d{1,2}$").unwrap();
        let regex_cpu_usage_last = regex::Regex::new(r"^CPU Core #\d{1,2}$").unwrap();
        let regex_disk_temperature_last = regex::Regex::new(r"^Temperature \d{1,2}$").unwrap();
        for sensor in sensors {
            if sensor.name == "CPU Package" && sensor.info == "Temperature" {
                map.insert("cpu_temperature".to_string(), sensor.index);
            } else if sensor.name == "CPU Core #1" && sensor.info == "Temperature" {
                map.insert("cpu_temperature_first".to_string(), sensor.index);
            } else if regex_cpu_temperature_last.is_match(&sensor.name) && sensor.info == "Temperature" {
                map.insert("cpu_temperature_last".to_string(), sensor.index);
            } else if sensor.name == "CPU Core #1 Distance to TjMax" && sensor.info == "Temperature" {
                map.insert("cpu_tjmax_first".to_string(), sensor.index);
            } else if regex_cpu_tjmax_last.is_match(&sensor.name) && sensor.info == "Temperature" {
                map.insert("cpu_tjmax_last".to_string(), sensor.index);
            } else if sensor.name == "CPU Package" && sensor.info == "Power" {
                map.insert("cpu_power".to_string(), sensor.index);
            } else if sensor.name == "CPU Core #1" && sensor.info == "Voltage" {
                map.insert("cpu_voltage_first".to_string(), sensor.index);
            } else if regex_cpu_voltage_last.is_match(&sensor.name) && sensor.info == "Voltage" {
                map.insert("cpu_voltage_last".to_string(), sensor.index);
            } else if sensor.name == "CPU Core" && sensor.info == "Voltage" {
                map.insert("cpu_voltage".to_string(), sensor.index);
            } else if sensor.name == "CPU Core #1" && sensor.info == "Clock" {
                map.insert("cpu_clock_first".to_string(), sensor.index);
            } else if regex_cpu_clock_last.is_match(&sensor.name) && sensor.info == "Clock" {
                map.insert("cpu_clock_last".to_string(), sensor.index);
            } else if sensor.name == "CPU Total" && sensor.info == "Load" {
                map.insert("cpu_usage".to_string(), sensor.index);
            } else if sensor.name == "CPU Core #1" && sensor.info == "Load" {
                map.insert("cpu_usage_first".to_string(), sensor.index);
            } else if regex_cpu_usage_last.is_match(&sensor.name) && sensor.info == "Load" {
                map.insert("cpu_usage_last".to_string(), sensor.index);
            } else if sensor.name == "GPU Core" && sensor.info == "Temperature" {
                map.insert("gpu_temperature".to_string(), sensor.index);
            } else if sensor.name == "GPU Package" && sensor.info == "Power" {
                map.insert("gpu_power".to_string(), sensor.index);
            } else if sensor.name == "GPU Core" && sensor.info == "Clock" {
                map.insert("gpu_clock_rms".to_string(), sensor.index);
            } else if sensor.name == "GPU Memory" && sensor.info == "Clock" {
                map.insert("gpu_mem_clock_rms".to_string(), sensor.index);
            } else if sensor.name == "GPU Core" && sensor.info == "Load" {
                map.insert("gpu_usage".to_string(), sensor.index);
            } else if sensor.name == "Memory" && sensor.info == "Load" {
                map.insert("mem_percentage".to_string(), sensor.index);
            } else if sensor.name == "Memory Available" && sensor.info == "Data" {
                map.insert("mem_available".to_string(), sensor.index);
            } else if sensor.name == "Memory Used" && sensor.info == "Data" {
                map.insert("mem_used".to_string(), sensor.index);
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
            } else if sensor.name == "Temperature 1" && sensor.info == "Temperature" {
                map.insert("disk_temperature_first".to_string(), sensor.index);
            } else if regex_disk_temperature_last.is_match(&sensor.name) && sensor.info == "Temperature" {
                map.insert("disk_temperature_last".to_string(), sensor.index);
            }
        }
        // THE CODE ABOVE IS SCRIPT GENERATED, DON'T CHANGE THEM DIRECTLY! CHANGE THE SCRIPT sensor_map.py INSTEAD

        map
    }

    fn query_sensor_value(&mut self, index: i32) -> Result<f64> {
        let value = self.lhm_helper.get_value(index)?;

        Ok(value)
    }

    fn set_battery_state(&mut self, map: &mut HashMap<&str, Option<DataContainer>>) {
        let capacity = match map.get("bat_capacity_remain").and_then(|v| v.as_ref()) {
            Some(DataContainer::Float(v)) => Some(*v),
            _ => None,
        };
        let rate = match map.get("bat_rate").and_then(|v| v.as_ref()) {
            Some(DataContainer::Float(v)) => Some(*v),
            _ => None,
        };

        if let (Some(capacity), Some(rate)) = (capacity, rate) {
            let prev_capacity = self.payload.prev_bat_capacity;

            if prev_capacity < capacity || rate == 0.0 {
                map.insert("bat_state", Some(DataContainer::Boolean(true)));
            } else if prev_capacity > capacity {
                map.insert("bat_state", Some(DataContainer::Boolean(false)));
            };

            self.payload.prev_bat_capacity = capacity;
        }
    }

    fn set_cpu_clock(&mut self, map: &mut HashMap<&str, Option<DataContainer>>) -> Result<()> {
        let clock_begin = *self.index_map.get("cpu_clock_first").unwrap();
        let clock_end = *self.index_map.get("cpu_clock_last").unwrap();

        if clock_begin != -1 && clock_end != -1 {
            let mut clocks = Vec::new();
            for index in clock_begin..=clock_end {
                let value = self.query_sensor_value(index).map_err(|e| anyhow!(e))?;
                clocks.push(value);
            }
            let clock_avg = clocks.iter().sum::<f64>() / clocks.len() as f64;
            map.insert(
                "cpu_clock_avg",
                if !clocks.is_empty() {
                    Some(DataContainer::Float(clock_avg))
                } else {
                    None
                },
            );
            map.insert(
                "cpu_clock_max",
                if !clocks.is_empty() {
                    Some(DataContainer::Float(
                        *clocks
                            .iter()
                            .max_by(|a, b| a.partial_cmp(b).unwrap())
                            .unwrap(),
                    ))
                } else {
                    None
                },
            );
            map.insert(
                "cpu_clock_rms",
                if clocks.len() > 3 {
                    clocks.sort_by(|a, b| a.partial_cmp(b).unwrap());
                    clocks.reverse();
                    let rms;
                    if clocks[0] - clocks[1] > 500.0 {
                        rms =
                            (clocks[0] * 0.3 + clocks[1] * 0.4 + clocks[2] * 0.2 + clocks[3] * 0.1)
                                / 1000.0;
                    } else {
                        rms = (clocks[0] * 0.35
                            + clocks[1] * 0.35
                            + clocks[2] * 0.2
                            + clocks[3] * 0.1)
                            / 1000.0;
                    }
                    Some(DataContainer::Float(rms))
                } else if clocks.is_empty() {
                    None
                } else {
                    Some(DataContainer::Float(clock_avg))
                },
            );
        }
        Ok(())
    }

    fn set_activation_state(&mut self, map: &mut HashMap<&str, Option<DataContainer>>) {
        let mut slmgr = ExternalProgram::new_transient(
            "cscript".to_string(),
            ProgramKind::Command,
            vec![vec![
                "//NoLogo".to_string(),
                "C:\\Windows\\System32\\slmgr.vbs".to_string(),
                "/xpr".to_string(),
            ]],
        );
        match slmgr.start(0) {
            Ok(output) => {
                if output.contains("permanently activated") || output.contains("计算机已永久激活") {
                    map.insert("os_activated", Some(DataContainer::Boolean(true)));
                } else {
                    map.insert("os_activated", Some(DataContainer::Boolean(false)));
                }
            }
            Err(e) => {
                debug!("Failed to query OS activation status with error {:?}", e);
            }
        }
    }

    fn set_disk_info(&mut self, map: &mut HashMap<&str, Option<DataContainer>>) {
        let mut diskpart = ExternalProgram::new_interpreter(
            "diskpart".to_string(),
            ProgramKind::Command,
            vec![vec![]],
        );
        if let Err(e) = diskpart.start(0) {
            debug!("Failed to start diskpart with error {:?}", e);
        } else {
            diskpart
                .consume_initial_output("DISKPART>".to_string())
                .unwrap();

            match diskpart.interact("list disk".to_string(), Some("DISKPART>".to_string())) {
                Ok(output) => {
                    let mut res = Vec::new();
                    let reg = Regex::new(r"\s+").unwrap();
                    let output: Vec<&str> = output.trim().split("\n").collect();
                    for i in 2..output.len() {
                        // ignore title lines
                        let line: Vec<&str> = reg.split(output[i].trim()).collect();
                        if line[0] == "Disk" && line.len() >= 4 {
                            // so it is disk info line
                            res.push(line[3]);
                        }
                    }

                    map.insert(
                        "disk_disk_size",
                        if !res.is_empty() {
                            let sizes: Vec<DataContainer> = res
                                .iter()
                                .map(|s| DataContainer::from(s.parse::<f64>().unwrap()))
                                .collect();
                            Some(DataContainer::Array(sizes))
                        } else {
                            None
                        },
                    );
                }
                Err(e) => {
                    debug!("Failed to query disk information with error {:?}", e);
                }
            }
        }
        diskpart.close();
    }
}

#[cfg(all(windows, test))]
mod tests {
    use super::*;

    #[test]
    fn test_match() {
        let windows = Windows::build().unwrap();
        let windows = windows.lock().unwrap();
        assert_ne!(
            windows.index_map.get("cpu_usage_first").unwrap(),
            windows.index_map.get("cpu_usage_last").unwrap()
        );
        assert_ne!(
            windows.index_map.get("cpu_temperature_first").unwrap(),
            windows.index_map.get("cpu_temperature_last").unwrap()
        );
    }
}
