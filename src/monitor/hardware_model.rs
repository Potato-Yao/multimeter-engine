// #![allow(dead_code)]

#[derive(Default, Debug, Clone)]
pub struct Device {
    pub battery: Battery,
    pub cpu: CPU,
    pub gpu: GPU,
    pub ram: RAM,
    pub fans: Fans,
    pub motherboard: Motherboard,
    pub disk: Disk,
    pub network: Network,
}

#[derive(Default, Debug, Clone)]
pub struct Battery {
    pub designed_capacity: Option<f64>, // the maximum capacity the battery should have
    pub actually_capacity: Option<f64>, // the maximum capacity the battery actually has
    pub remain_capacity: Option<f64>,   // the current remaining capacity
    pub voltage: Option<f64>,           // the voltage battery supplies
    pub current: Option<f64>,           // the current battery supplies
    pub rate: Option<f64>,              // charge or discharge power
    pub is_charging: Option<bool>,      // charging state
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
#[derive(Default, Debug, Clone)]
pub struct CPU {
    pub name: Option<String>,             // name of the CPU
    pub usage: Option<f64>,               // cpu usage
    pub package_temperature: Option<f64>, // package temperature
    pub average_temperature: Option<f64>, // average core temperature
    pub power: Option<f64>,               // power consumption
    pub clock_begin_index: Option<i32>,   // sensor index start
    pub clock_end_index: Option<i32>,     // sensor index end
    pub clock: Option<f64>,               // calculated clock speed
    pub load: Option<f64>,                // cpu load
    pub voltage: Option<f64>,             // cpu voltage
}

#[allow(clippy::upper_case_acronyms)]
#[derive(Default, Debug, Clone)]
pub struct GPU {
    pub name: Option<String>,
    pub temperature: Option<f64>,
    pub max_temperature: Option<f64>,
    pub power: Option<f64>,
    pub speed: Option<f64>,
    pub mem_total: Option<f64>,
    pub mem_free: Option<f64>,
    pub mem_used: Option<f64>,
    pub mem_usage: Option<f64>,
}

#[allow(clippy::upper_case_acronyms)]
#[derive(Default, Debug, Clone)]
pub struct RAM {
    // the reason used, free and total all here, even total can be calculated by the other two is some api gives a total, which is more accuracy than the sum of free and used
    pub used_size: Option<f64>,
    pub free_size: Option<f64>,
    pub total_size: Option<f64>,
    pub total_swap: Option<f64>,
    pub used_swap: Option<f64>,
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

#[derive(Default, Debug, Clone)]
pub struct Fans {
    pub fan_speed: Option<Vec<i32>>,
}

#[derive(Default, Debug, Clone)]
pub struct Motherboard {
    pub name: Option<String>,
}

#[derive(Default, Debug, Clone)]
pub struct Disk;

#[derive(Default, Debug, Clone)]
pub struct Network;
