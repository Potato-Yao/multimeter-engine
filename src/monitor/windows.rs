use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use anyhow::Result;

use crate::external_program::lhm_helper::LhmHelper;
use crate::monitor::hardware_model::{Battery, CPU, GPU, RAM};
use crate::util::admin::is_admin;
use serde::Deserialize;
use crate::monitor::{QueryRequest, Queryable};
use crate::util::payload::PayLoad;

#[cfg(windows)]
#[derive(Debug, Clone, Deserialize)]
struct Sensor {
    name: String,
    info: String,
    index: i32,
}

#[cfg(windows)]
pub struct Windows {
    index: [i32; 256],
    cpu: CPU,
    gpu: GPU,
    ram: RAM,
    battery: Battery,
    lhm_helper: LhmHelper,
    prev_battery_capacity: f64,
}

#[cfg(windows)]
impl Queryable for Windows {
    fn query(&self, request: &QueryRequest) -> Result<PayLoad> {
        todo!()
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

        let mut manager = Windows {
            index: [-1; 256],
            cpu: CPU::default(),
            gpu: GPU::default(),
            ram: RAM::default(),
            battery: Battery::default(),
            lhm_helper,
            prev_battery_capacity: -1.0,
        };

        let mapping_table = Self::get_sensor_mapping_lookup();

        for sensor in sensors {
            if let Some(&target_idx) =
                mapping_table.get(&(sensor.name.as_str(), sensor.info.as_str()))
            {
                manager.index[target_idx] = sensor.index;
            } else if sensor.name.contains("CPU Core #") && sensor.info == "Clock" {
                manager.index[6] = manager.index[6].max(sensor.index);
            }
        }

        let manager_arc = Arc::new(Mutex::new(manager));
        let manager_clone = Arc::clone(&manager_arc);

        thread::spawn(move || {
            loop {
                {
                    let mut mgr = manager_clone.lock().unwrap();
                    if let Err(e) = mgr.update() {
                        eprintln!("Update failed: {}", e);
                        break;
                    }
                }
                thread::sleep(Duration::from_millis(500));
            }
        });

        Ok(manager_arc)
    }

    fn get_sensor_mapping_lookup() -> HashMap<(&'static str, &'static str), usize> {
        let mut m = HashMap::new();
        m.insert(("CPU Total", "Load"), 0);
        m.insert(("CPU Package", "Temperature"), 1);
        m.insert(("Core Average", "Temperature"), 2);
        m.insert(("CPU Package", "Power"), 3);
        m.insert(("CPU Core", "Voltage"), 4);
        m.insert(("CPU Core #1", "Clock"), 5);
        m.insert(("GPU Core", "Temperature"), 34);
        m.insert(("GPU Hot Spot", "Temperature"), 35);
        m.insert(("GPU Package", "Power"), 36);
        m.insert(("GPU Core", "Clock"), 37);
        m.insert(("GPU Memory Total", "SmallData"), 38);
        m.insert(("GPU Memory Free", "SmallData"), 39);
        m.insert(("GPU Memory Used", "SmallData"), 40);
        m.insert(("Memory Used", "Data"), 65);
        m.insert(("Memory Available", "Data"), 66);
        m.insert(("Fully-Charged Capacity", "Energy"), 113);
        m.insert(("Remaining Capacity", "Energy"), 114);
        m.insert(("Voltage", "Voltage"), 115);
        let battery_fields = [
            "Charge Current",
            "Discharge Current",
            "Charge/Discharge Current",
        ];
        for field in battery_fields {
            m.insert((field, "Current"), 116);
        }

        let rate_fields = ["Charge Rate", "Discharge Rate", "Charge/Discharge Rate"];
        for field in rate_fields {
            m.insert((field, "Power"), 117);
        }

        m.insert(("Designed Capacity", "Energy"), 118);
        m
    }

    pub fn update(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.lhm_helper.update()?;

        todo!();

        Ok(())
    }

    fn get_clock_value(&mut self, begin: i32, end: i32) -> f64 {
        if begin == -1 || end == -1 {
            return 0.0;
        }

        let mut clocks = [0.0f64; 4]; // Keep track of top 4 clocks
        for i in begin..=end {
            let val = self.lhm_helper.get_value(i).unwrap();
            if val > clocks[0] {
                clocks[3] = clocks[2];
                clocks[2] = clocks[1];
                clocks[1] = clocks[0];
                clocks[0] = val;
            }
        }

        let mut clock = if clocks[0] - clocks[1] > 500.0 {
            clocks[0] * 0.3 + clocks[1] * 0.4 + clocks[2] * 0.2 + clocks[3] * 0.1
        } else {
            clocks[0] * 0.35 + clocks[1] * 0.35 + clocks[2] * 0.2 + clocks[3] * 0.1
        };

        clock / 1000.0
    }

    fn set_charging_state(&mut self) {
        let current = self.battery.remain_capacity;
        if self.prev_battery_capacity < current || self.battery.rate == 0.0 {
            self.battery.is_charging = true;
        } else if self.prev_battery_capacity > current {
            self.battery.is_charging = false;
        }
        self.prev_battery_capacity = current;
    }
}
