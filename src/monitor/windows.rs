use crate::external_program::lhm_helper::LhmHelper;
use crate::external_program::program::Program;
use crate::monitor::model::Device;
use crate::monitor::{QUERY_STATEMENTS, Updater};
use crate::util::admin::is_admin;
use anyhow::{Result, anyhow};
use log::{debug, trace};
use serde::Deserialize;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

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
    fn update_once(&mut self, device: &mut Device) -> Result<()> {
        self.set_activation_state(device)?;

        Ok(())
    }

    fn update_slow(&mut self, device: &mut Device) -> Result<()> {
        self.set_disk_info(device)?;

        Ok(())
    }

    fn update(&mut self, device: &mut Device) -> Result<()> {
        self.lhm_helper.update()?;

        if let Some(value) = self.query_optional_sensor_value("bat_capacity_designed")? {
            device.battery.designed_capacity = Some(value);
        }
        if let Some(value) = self.query_optional_sensor_value("bat_capacity_max")? {
            device.battery.actually_capacity = Some(value);
        }
        if let Some(value) = self.query_optional_sensor_value("bat_capacity_remain")? {
            device.battery.remain_capacity = Some(value);
        }
        if let Some(value) = self.query_optional_sensor_value("bat_voltage")? {
            device.battery.voltage = Some(value);
        }
        if let Some(value) = self.query_optional_sensor_value("bat_rate")? {
            device.battery.rate = Some(value);
        }

        if let Some(value) = self.query_optional_sensor_value("cpu_usage")? {
            device.cpu.usage = Some(value);
        }
        if let Some(value) = self.query_optional_sensor_value("cpu_temperature")? {
            device.cpu.package_temperature = Some(value);
        }
        if let Some(value) = self.query_optional_sensor_value("cpu_power")? {
            device.cpu.power = Some(value);
        }
        if let Some(value) = self.query_optional_sensor_value("cpu_voltage")? {
            device.cpu.voltage = Some(value);
        }

        if let Some(value) = self.query_optional_sensor_value("gpu_temperature")? {
            device.gpu.temperature = Some(value);
        }
        if let Some(value) = self.query_optional_sensor_value("gpu_power")? {
            device.gpu.power = Some(value);
        }
        if let Some(value) = self.query_optional_sensor_value("gpu_clock_rms")? {
            device.gpu.speed = Some(value);
        }
        if let Some(value) = self.query_optional_sensor_value("gpu_mem_clock_rms")? {
            device.gpu.mem_clock = Some(value as i32);
        }

        if let Some(value) = self.query_optional_sensor_value("mem_available")? {
            device.ram.free_size = Some(value);
        }
        if let Some(value) = self.query_optional_sensor_value("mem_used")? {
            device.ram.used_size = Some(value);
        }

        self.set_battery_state(device);
        self.set_cpu_clock(device)?;

        Ok(())
    }

    fn shutdown(&mut self) -> Result<()> {
        if let Err(e) = self.lhm_helper.disconnect() {
            return Err(anyhow!(e));
        }

        Ok(())
    }
}

impl Windows {
    pub fn build() -> Result<Arc<Mutex<Self>>> {
        if !is_admin() {
            return Err(anyhow::anyhow!("Windows is not admin"));
        }

        let mut lhm_helper = LhmHelper::connect()?;
        let lhm_sensors_json = lhm_helper.query_hardware()?;

        // let mut path = env::current_dir().unwrap();
        // path.push("doc");
        // path.push("hardware_lists");
        // // path.push("i7-14650HX_RTX5060.json");
        // // path.push("ultra7155H_RTX4060.json");
        // // path.push("ultra9185H_RTX4060.json");
        // path.push("R78745H_R780M.json");
        // debug!("reading {:?}", path);
        // let lhm_sensors_json = read_to_string(path)?;

        let sensors: Vec<Sensor> = serde_json::from_str(&lhm_sensors_json)?;
        let index_map = Self::build_index_map(&sensors);
        trace!("The index map: {:?}", index_map);

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
        let regex_cpu_clock_last = regex::Regex::new(r"^Core #\d{1,2}$").unwrap();
        let regex_cpu_usage_last = regex::Regex::new(r"^CPU Core #\d{1,2}$").unwrap();
        let regex_disk_temperature_last = regex::Regex::new(r"^Temperature \d{1,2}$").unwrap();
        for sensor in sensors {
            if sensor.name == "CPU Package" && sensor.info == "Temperature" {
                map.insert("cpu_temperature".to_string(), sensor.index);
            } else if sensor.name == "Core (Tctl/Tdie)" && sensor.info == "Temperature" {
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
            } else if sensor.name == "Package" && sensor.info == "Power" {
                map.insert("cpu_power".to_string(), sensor.index);
            } else if sensor.name == "CPU Core #1" && sensor.info == "Voltage" {
                map.insert("cpu_voltage_first".to_string(), sensor.index);
            } else if regex_cpu_voltage_last.is_match(&sensor.name) && sensor.info == "Voltage" {
                map.insert("cpu_voltage_last".to_string(), sensor.index);
            } else if sensor.name == "CPU Core" && sensor.info == "Voltage" {
                map.insert("cpu_voltage".to_string(), sensor.index);
            } else if sensor.name == "Core (SVI2 TFN)" && sensor.info == "Voltage" {
                map.insert("cpu_voltage".to_string(), sensor.index);
            } else if sensor.name == "CPU Core #1" && sensor.info == "Clock" {
                map.insert("cpu_clock_first".to_string(), sensor.index);
            } else if sensor.name == "Core #1" && sensor.info == "Clock" {
                map.insert("cpu_clock_first".to_string(), sensor.index);
            } else if regex_cpu_clock_last.is_match(&sensor.name) && sensor.info == "Clock" {
                map.insert("cpu_clock_last".to_string(), sensor.index);
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
            } else if sensor.name == "GPU VR SoC" && sensor.info == "Temperature" {
                map.insert("gpu_temperature".to_string(), sensor.index);
            } else if sensor.name == "GPU Package" && sensor.info == "Power" {
                map.insert("gpu_power".to_string(), sensor.index);
            } else if sensor.name == "GPU Core" && sensor.info == "Power" {
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

    fn query_optional_sensor_value(&mut self, key: &str) -> Result<Option<f64>> {
        if let Some(index) = self.index_map.get(key)
            && *index != -1
        {
            return Ok(Some(
                self.query_sensor_value(*index).map_err(|e| anyhow!(e))?,
            ));
        }

        Ok(None)
    }

    fn set_battery_state(&mut self, device: &mut Device) {
        let capacity = device.battery.remain_capacity;
        let rate = device.battery.rate;

        if let (Some(capacity), Some(rate)) = (capacity, rate) {
            let prev_capacity = self.payload.prev_bat_capacity;

            if prev_capacity < capacity || rate == 0.0 {
                device.battery.is_charging = Some(true);
            } else if prev_capacity > capacity {
                device.battery.is_charging = Some(false);
            };

            self.payload.prev_bat_capacity = capacity;
        }
    }

    fn set_cpu_clock(&mut self, device: &mut Device) -> Result<()> {
        let clock_begin = *self.index_map.get("cpu_clock_first").unwrap();
        let clock_end = *self.index_map.get("cpu_clock_last").unwrap();

        if clock_begin != -1 && clock_end != -1 {
            let mut clocks = Vec::new();
            for index in clock_begin..=clock_end {
                let value = self.query_sensor_value(index).map_err(|e| anyhow!(e))?;
                clocks.push(value);
            }
            let clock_avg = clocks.iter().sum::<f64>() / clocks.len() as f64;

            if clocks.len() > 3 {
                clocks.sort_by(|a, b| a.partial_cmp(b).unwrap());
                clocks.reverse();
                let rms;
                if clocks[0] - clocks[1] > 500.0 {
                    rms = (clocks[0] * 0.3 + clocks[1] * 0.4 + clocks[2] * 0.2 + clocks[3] * 0.1)
                        / 1000.0;
                } else {
                    rms = (clocks[0] * 0.35 + clocks[1] * 0.35 + clocks[2] * 0.2 + clocks[3] * 0.1)
                        / 1000.0;
                }
                device.cpu.clock = Some(rms);
            } else if !clocks.is_empty() {
                device.cpu.clock = Some(clock_avg);
            }
        }
        Ok(())
    }

    fn set_activation_state(&mut self, device: &mut Device) -> Result<()> {
        let mut slmgr = Program::new_command(
            "cscript",
            Some(vec![vec![
                "//NoLogo".to_string(),
                "C:\\Windows\\System32\\slmgr.vbs".to_string(),
                "/xpr".to_string(),
            ]]),
        );

        slmgr.start(Some(0))?;

        if slmgr.read()?.contains("permanently activated")
            || slmgr.read()?.contains("计算机已永久激活")
        {
            device.system.is_activated = Some(true);
        } else {
            device.system.is_activated = Some(false);
        }

        Ok(())
    }

    fn set_disk_info(&mut self, _device: &mut Device) -> Result<()> {
        let mut diskpart = Program::new_command("diskpart", None);

        diskpart.start(None)?;

        diskpart.read()?;
        diskpart.write("list disk")?;

        // diskpart
        //     .consume_initial_output("DISKPART>".to_string())
        //     .unwrap();

        // match diskpart.interact("list disk".to_string(), Some("DISKPART>".to_string())) {
        //     Ok(output) => {
        //         let mut res = Vec::new();
        //         let reg = Regex::new(r"\s+").unwrap();
        //         let output: Vec<&str> = output.trim().split("\n").collect();
        //         for i in 2..output.len() {
        //             // ignore title lines
        //             let line: Vec<&str> = reg.split(output[i].trim()).collect();
        //             if line[0] == "Disk" && line.len() >= 4 {
        //                 // so it is disk info line
        //                 res.push(line[3]);
        //             }
        //         }
        //
        //         map.insert(
        //             "disk_disk_size",
        //             if !res.is_empty() {
        //                 let sizes: Vec<DataContainer> = res
        //                     .iter()
        //                     .map(|s| DataContainer::from(s.parse::<f64>().unwrap()))
        //                     .collect();
        //                 Some(DataContainer::Array(sizes))
        //             } else {
        //                 None
        //             },
        //         );
        //     }
        //     Err(e) => {
        //         debug!("Failed to query disk information with error {:?}", e);
        //     }
        // }
        diskpart.close()?;

        Ok(())
    }
}

#[cfg(test)]
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
