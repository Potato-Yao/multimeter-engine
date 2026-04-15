use crate::insert_data;
use crate::monitor::Updater;
use crate::util::data_container::DataContainer;
use anyhow::Result;
use lm_sensors::LMSensors;
use nvml_wrapper::Nvml;
use nvml_wrapper::enum_wrappers::device::{Clock, TemperatureSensor};
use std::collections::HashMap;
use std::fs::read_to_string;
use std::sync::{Arc, Mutex};
use std::time::Instant;

struct SensorWrapper(LMSensors, Option<Nvml>);

unsafe impl Send for SensorWrapper {}
unsafe impl Sync for SensorWrapper {}

struct PowerCalculator(Instant, i128);

unsafe impl Send for PowerCalculator {}
unsafe impl Sync for PowerCalculator {}

impl PowerCalculator {
    fn build() -> Result<Self> {
        Ok(Self(Instant::now(), Self::read_energy_consume()?))
    }

    fn calculate(&mut self) -> Result<f64> {
        let now_energy = Self::read_energy_consume()?;
        let now = Instant::now();
        let power = (now_energy - self.1) as f64 / now.duration_since(self.0).as_secs_f64() / 1_000_000.0; // power = work / time, converted to watt
        self.0 = now;
        self.1 = now_energy;

        Ok(power)
    }

    /// the counter gives energy consumed by muJ. if you want to convert it to J, divide 1_000_000
    fn read_energy_consume() -> Result<i128> {
        let path = "/sys/class/powercap/intel-rapl:0/energy_uj";
        let raw = read_to_string(path)?;
        Ok(raw.trim().parse::<i128>()?)
    }
}

enum State {
    TEMP,
    FAN,
    BAT,
}

pub struct Linux {
    sensor: SensorWrapper,
    power_calculator: PowerCalculator,
}

#[cfg(target_os = "linux")]
impl Updater for Linux {
    fn update_once(&mut self, map: &mut HashMap<&str, Option<DataContainer>>) -> Result<()> {
        insert_data!(map, "os_activated", true); // Linux is free os

        Ok(())
    }

    fn update_slow(&mut self, map: &mut HashMap<&str, Option<DataContainer>>) -> Result<()> {
        Ok(())
    }

    fn update(&mut self, map: &mut HashMap<&str, Option<DataContainer>>) -> Result<()> {
        self.set_libsensor_info(map)?;
        self.set_nvml_info(map)?;
        self.set_cpu_power(map)?;

        Ok(())
    }

    fn shutdown(&mut self) -> Result<()> {
        Ok(())
    }
}

#[cfg(target_os = "linux")]
impl Linux {
    pub fn build() -> Result<Arc<Mutex<Self>>> {
        let lm_sensor = lm_sensors::Initializer::default().initialize()?;
        let nvml = match Nvml::init() {
            Ok(n) => Some(n),
            _ => None,
        };

        Ok(Arc::new(Mutex::new(Self {
            sensor: SensorWrapper(lm_sensor, nvml),
            power_calculator: PowerCalculator::build()?,
        })))
    }

    fn set_nvml_info(&mut self, map: &mut HashMap<&str, Option<DataContainer>>) -> Result<()> {
        if let Some(sensor) = &self.sensor.1 {
            let device = sensor.device_by_index(0)?;
            insert_data!(map, "gpu_name", device.name()?);
            insert_data!(map, "gpu_power", device.power_usage()? as f64 / 1000.0);
            insert_data!(
                map,
                "gpu_temperature",
                device.temperature(TemperatureSensor::Gpu)? as i32
            );
            insert_data!(
                map,
                "gpu_clock_rms",
                device.clock_info(Clock::Graphics)? as i32
            );
            insert_data!(
                map,
                "gpu_mem_clock_rms",
                device.clock_info(Clock::Memory)? as i32
            );
        }

        Ok(())
    }

    fn set_libsensor_info(&mut self, map: &mut HashMap<&str, Option<DataContainer>>) -> Result<()> {
        let sensors = &self.sensor.0;
        let mut state: Option<State> = None;

        for chip in sensors.chip_iter(None) {
            if chip.to_string().contains("coretemp") {
                state = Some(State::TEMP);
            // } else if chip.to_string().contains("BAT") {
            //     state = Some(State::BAT);
            } else {
                if let Some(feature) = chip.feature_iter().next() {
                    if feature.to_string().contains("fan") {
                        state = Some(State::FAN);
                    }
                }
            }

            match state {
                Some(State::TEMP) => {
                    'outer: for feature in chip.feature_iter() {
                        if feature.to_string().contains("Package") {
                            for sub in feature.sub_feature_iter() {
                                if sub.to_string().contains("input") {
                                    let value = sub.value()?.to_string();
                                    insert_data!(
                                        map,
                                        "cpu_temperature",
                                        remove_unit(&value).parse::<i32>()?
                                    );
                                    break 'outer;
                                }
                            }
                        }
                    }
                }
                Some(State::FAN) => {
                    let fan_types = [
                        ("cpu", "fan_rpm_cpu"),
                        ("gpu", "fan_rpm_gpu"),
                        ("mid", "fan_rpm_mid"),
                    ];
                    for feature in chip.feature_iter() {
                        let feature_name = feature.to_string();
                        for &(pattern, key) in &fan_types {
                            if feature_name.contains(pattern) {
                                for sub in feature.sub_feature_iter() {
                                    if sub.to_string().contains("input") {
                                        let value = sub.value()?.to_string();
                                        insert_data!(map, key, remove_unit(&value).parse::<i32>()?);
                                        break;
                                    }
                                }
                            }
                        }
                    }
                }
                Some(State::BAT) => {
                    //     let mut current: f64 = -1.0;
                    //     let mut voltage: f64 = -1.0;
                    //     for feature in chip.feature_iter() {
                    //         if feature.to_string().contains("in") {
                    //             if let Some(sub) = feature.sub_feature_iter().next() {
                    //                 voltage = remove_unit(&sub.value()?.to_string()).parse()?;
                    //             }
                    //         } else if feature.to_string().contains("curr") {
                    //             if let Some(sub) = feature.sub_feature_iter().next() {
                    //                 current = remove_unit(&sub.value()?.to_string()).parse()?;
                    //             }
                    //         }
                    //     }
                    //     if voltage != -1.0 {
                    //         insert_data!(map, "bat_voltage", voltage);
                    //
                    //         if current != -1.0 {
                    //             insert_data!(map, "bat_rate", current * voltage);
                    //         }
                    //     }
                }
                None => {}
            }
        }
        Ok(())
    }

    fn set_cpu_power(&mut self, map: &mut HashMap<&str, Option<DataContainer>>) -> Result<()> {
        let power = self.power_calculator.calculate()?;
        insert_data!(map, "cpu_power", power);

        Ok(())
    }
}

fn remove_unit(string: &str) -> &str {
    string.split(' ').collect::<Vec<&str>>()[0]
}

#[cfg(all(target_os = "linux", test))]
mod tests {
    use crate::monitor::linux::remove_unit;
    use nvml_wrapper::Nvml;
    use nvml_wrapper::enum_wrappers::device::TemperatureSensor;

    #[test]
    fn test_libsensor() {
        let sensors = lm_sensors::Initializer::default().initialize().unwrap();

        for chip in sensors.chip_iter(None) {
            if let Some(path) = chip.path() {
                println!("chip: {} at {} ({})", chip, chip.bus(), path.display());
            } else {
                println!("chip: {} at {}", chip, chip.bus());
            }

            for feature in chip.feature_iter() {
                let name = feature.name().transpose().unwrap().unwrap_or("N/A");
                println!("    {}: {}", name, feature);

                for sub_feature in feature.sub_feature_iter() {
                    if let Ok(value) = sub_feature.value() {
                        println!("        {}: {}", sub_feature, value);
                    } else {
                        println!("        {}: N/A", sub_feature);
                    }
                }
            }
        }
    }

    #[test]
    fn test_nvidia() {
        let nvml = Nvml::init().unwrap();
        let device = nvml.device_by_index(0).unwrap();
        println!("{}", device.name().unwrap());
        println!("{}", device.num_fans().unwrap());
        println!("{:?}", device.memory_info().unwrap());
        println!("{}", device.temperature(TemperatureSensor::Gpu).unwrap());
    }

    #[test]
    fn test_unit_remove() {
        assert_eq!("2400", remove_unit("2400 RPM"));
        assert_eq!("65", remove_unit("65 C"));
    }
}
