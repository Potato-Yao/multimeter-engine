#![allow(dead_code)]
pub trait HardwareItem {}

#[derive(Default, Debug, Clone)]
pub struct Battery {
    pub designed_capacity: f64, // the maximum capacity the battery should have
    pub actually_capacity: f64, // the maximum capacity the battery actually has
    pub remain_capacity: f64,   // the current remaining capacity
    pub voltage: f64,           // the voltage battery supplies
    pub current: f64,           // the current battery supplies
    pub rate: f64,              // charge or discharge power
    pub is_charging: bool,      // charging state
}

impl Battery {
    pub fn get_health_percentage(&self) -> f64 {
        if self.designed_capacity == 0.0 {
            return 0.0;
        }
        self.actually_capacity / self.designed_capacity
    }

    pub fn get_remain_percentage(&self) -> f64 {
        if self.actually_capacity == 0.0 {
            return 0.0;
        }
        (self.remain_capacity / self.actually_capacity) * 100.0
    }
}

#[derive(Default, Debug, Clone)]
pub struct CPU {
    pub name: String,             // name of the CPU
    pub usage: f64,               // cpu usage
    pub package_temperature: f64, // package temperature
    pub average_temperature: f64, // average core temperature
    pub power: f64,               // power consumption
    pub clock_begin_index: i32,   // sensor index start
    pub clock_end_index: i32,     // sensor index end
    pub clock: f64,               // calculated clock speed
    pub load: f64,                // cpu load
    pub voltage: f64,             // cpu voltage
}

#[derive(Default, Debug, Clone)]
pub struct GPU {
    pub name: String,
    pub temperature: f64,
    pub max_temperature: f64,
    pub power: f64,
    pub speed: f64,
    pub mem_total: f64,
    pub mem_free: f64,
    pub mem_used: f64,
    pub mem_usage: f64,
}

#[derive(Default, Debug, Clone)]
pub struct RAM {
    pub used_size: f64,
    pub free_size: f64,
}

impl RAM {
    pub fn get_total_size(&self) -> f64 {
        self.used_size + self.free_size
    }

    pub fn get_used_percentage(&self) -> f64 {
        let total = self.get_total_size();
        if total == 0.0 {
            return 0.0;
        }
        (self.used_size / total) * 100.0
    }
}

#[derive(Default, Debug, Clone)]
pub struct Fan {
    pub fan_speed: i32,
}

#[derive(Default, Debug, Clone)]
pub struct Motherboard {
    pub name: String,
}

#[derive(Default, Debug, Clone)]
pub struct Disk;

#[derive(Default, Debug, Clone)]
pub struct Network;

impl HardwareItem for Battery {}
impl HardwareItem for CPU {}
impl HardwareItem for GPU {}
impl HardwareItem for RAM {}
impl HardwareItem for Fan {}
impl HardwareItem for Motherboard {}
impl HardwareItem for Disk {}
impl HardwareItem for Network {}
