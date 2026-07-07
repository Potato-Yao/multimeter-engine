use crate::monitor::Updater;
use crate::monitor::model::Model;
use anyhow::{Result, anyhow};
use lm_sensors::{ChipRef, LMSensors};
use nvml_wrapper::Nvml;
use nvml_wrapper::enum_wrappers::device::{Clock, TemperatureSensor};
use std::fs::read_to_string;
use std::sync::{Arc, Mutex};
use std::time::Instant;

struct SensorWrapper(Option<LMSensors>, Option<Nvml>);

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
        let power =
            (now_energy - self.1) as f64 / now.duration_since(self.0).as_secs_f64() / 1_000_000.0; // power = work / time, converted to watt
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
    Temp,
    Fan,
}

pub struct Linux {
    sensor: SensorWrapper,
    power_calculator: Option<PowerCalculator>,
}

#[cfg(target_os = "linux")]
impl Updater for Linux {
    fn update_once(&mut self, device: &mut Model) -> Result<()> {
        device.system.is_activated = Some(true);

        Ok(())
    }

    fn update_slow(&mut self, _device: &mut Model) -> Result<()> {
        Ok(())
    }

    fn update(&mut self, device: &mut Model) -> Result<()> {
        let errors = [
            self.set_libsensor_info(device),
            self.set_nvml_info(device),
            self.set_cpu_power(device),
        ]
        .into_iter()
        .filter_map(|result| result.err().map(|err| err.to_string()))
        .collect::<Vec<_>>();

        if !errors.is_empty() {
            return Err(anyhow!("Linux update failed: {}", errors.join("; ")));
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
        let lm_sensor = lm_sensors::Initializer::default().initialize().ok();
        let nvml = Nvml::init().ok();
        let power_calculator = PowerCalculator::build().ok();

        Ok(Arc::new(Mutex::new(Self {
            sensor: SensorWrapper(lm_sensor, nvml),
            power_calculator,
        })))
    }

    fn set_nvml_info(&mut self, device: &mut Model) -> Result<()> {
        if self.sensor.1.is_none() {
            return Err(anyhow!("NVML sensor is unavailable"));
        }

        let sensor = self.sensor.1.as_ref().unwrap();
        let gpu = sensor.device_by_index(0)?;
        device.gpu.name = Some(gpu.name()?);
        device.gpu.power_usage = Some(gpu.power_usage()? as f64 / 1000.0);
        device.gpu.temperature = Some(gpu.temperature(TemperatureSensor::Gpu)? as f64);
        device.gpu.clock = Some(gpu.clock_info(Clock::Graphics)? as i32);
        device.gpu.mem_clock = Some(gpu.clock_info(Clock::Memory)? as i32);

        Ok(())
    }

    fn set_libsensor_info(&mut self, device: &mut Model) -> Result<()> {
        if self.sensor.0.is_none() {
            return Err(anyhow!("lm-sensors is unavailable"));
        }

        let sensors = self.sensor.0.as_ref().unwrap();
        let mut state: Option<State> = None;

        for chip in sensors.chip_iter(None) {
            if chip.to_string().contains("coretemp") {
                state = Some(State::Temp);
            // } else if chip.to_string().contains("BAT") {
            //     state = Some(State::BAT);
            } else {
                #[allow(clippy::collapsible_if)] // to fuck clippy
                if let Some(feature) = chip.feature_iter().next() {
                    if feature.to_string().contains("fan") {
                        state = Some(State::Fan);
                    }
                }
            }

            match state {
                Some(State::Temp) => {
                    Self::handle_temperature(device, chip)?;
                }
                Some(State::Fan) => {
                    Self::handle_fan(device, chip)?;
                }
                None => {}
            }
        }

        Ok(())
    }

    fn handle_fan(device: &mut Model, chip: ChipRef) -> Result<()> {
        for feature in chip.feature_iter() {
            let feature_name = feature.to_string();
            let feature_name = feature_name.as_str();

            if feature_name.contains("fan") {
                for sub in feature.sub_feature_iter() {
                    if sub.to_string().contains("input") {
                        let value = sub.value()?.to_string();
                        let value = remove_unit(&value).parse::<i32>()?;

                        match feature_name {
                            "cpu_fan" => {
                                device.fans.cpu_speed = Some(value);
                            }
                            "gpu_fan" => {
                                device.fans.gpu_speed = Some(value);
                            }
                            "mid_fan" => {
                                device.fans.mid_speed = Some(value);
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn handle_temperature(device: &mut Model, chip: ChipRef) -> Result<()> {
        'outer: for feature in chip.feature_iter() {
            if feature.to_string().contains("Package") {
                for sub in feature.sub_feature_iter() {
                    if sub.to_string().contains("input") {
                        let value = sub.value()?.to_string();
                        device.cpu.package_temperature = Some(remove_unit(&value).parse()?);
                        break 'outer;
                    }
                }
            }
        }

        Ok(())
    }

    fn set_cpu_power(&mut self, device: &mut Model) -> Result<()> {
        if self.power_calculator.is_none() {
            return Err(anyhow!(
                "CPU power calculator is unavailable, check sudo permission"
            ));
        }

        let power = self.power_calculator.as_mut().unwrap().calculate()?;
        device.cpu.usage = Some(power);

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
        let Ok(sensors) = lm_sensors::Initializer::default().initialize() else {
            return;
        };

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
