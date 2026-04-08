use crate::monitor::Updater;
use crate::util::data_container::DataContainer;
use anyhow::{Context, Result};
use lm_sensors::LMSensors;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

struct SensorWrapper(LMSensors);

unsafe impl Send for SensorWrapper {}
unsafe impl Sync for SensorWrapper {}

enum State {
    TEMP,
    FAN,
    BAT,
}

pub struct Linux {
    sensor: SensorWrapper,
}

#[cfg(target_os = "linux")]
impl Updater for Linux {
    fn update_once(&mut self, map: &mut HashMap<&str, Option<DataContainer>>) -> Result<()> {
        Ok(())
    }

    fn update_slow(&mut self, map: &mut HashMap<&str, Option<DataContainer>>) -> Result<()> {
        Ok(())
    }

    fn update(&mut self, map: &mut HashMap<&str, Option<DataContainer>>) -> Result<()> {
        let sensors = &self.sensor.0;
        let mut state: Option<State>;

        for chip in sensors.chip_iter(None) {
            if chip.to_string().contains("coretemp") {
                state = Some(State::TEMP);
            } else if chip.to_string().contains("BAT") {
                state = Some(State::BAT);
            } else {
                state = None;
            }

            match state {
                Some(State::TEMP) => {
                    'outer: for feature in chip.feature_iter() {
                        if feature.to_string().contains("Package") {
                            for sub in feature.sub_feature_iter() {
                                if sub.to_string().contains("input") {
                                    let value = sub.value()?.to_string();
                                    map.insert(
                                        "cpu_temperature",
                                        Some(DataContainer::Int(remove_unit(&value).parse()?)),
                                    );
                                    break 'outer;
                                }
                            }
                        }
                    }
                }
                Some(State::FAN) => {}
                Some(State::BAT) => {
                    let mut current: f64 = -1.0;
                    let mut voltage: f64 = -1.0;
                    for feature in chip.feature_iter() {
                        if feature.to_string().contains("in") {
                            if let Some(sub) = feature.sub_feature_iter().next() {
                                voltage = remove_unit(&sub.value()?.to_string()).parse()?;
                            }
                        } else if feature.to_string().contains("curr") {
                            if let Some(sub) = feature.sub_feature_iter().next() {
                                current = remove_unit(&sub.value()?.to_string()).parse()?;
                            }
                        }
                    }
                    if voltage != -1.0 {
                        map.insert("bat_voltage", Some(DataContainer::Float(voltage)));

                        if current != -1.0 {
                            map.insert("bat_rate", Some(DataContainer::Float(current * voltage)));
                        }
                    }
                }
                None => {}
            }
        }

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

        Ok(Arc::new(Mutex::new(Self {
            sensor: SensorWrapper(lm_sensor),
        })))
    }
}

fn remove_unit(string: &str) -> &str {
    &string[..string.len() - 2]
}

#[cfg(all(target_os = "linux", test))]
mod tests {
    use nvml_wrapper::Nvml;
    use nvml_wrapper::enum_wrappers::device::TemperatureSensor;

    #[test]
    fn test() {
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
}
