#[cfg(feature = "web-api")]
use super::INFO_MAP;
use crate::monitor::hardware_model::Device;
use crate::util::data_container::DataContainer;

#[derive(Default)]
pub struct HardwareManager {
    real_model: Device,
    pub temp_model: Device,
}

impl HardwareManager {
    pub fn commit(&mut self) {
        fn merge_option<T>(real: &mut Option<T>, temp: &Option<T>, _info_key: Option<&'static str>)
        where
            T: Clone + Into<DataContainer>,
        {
            if let Some(value) = temp {
                *real = Some(value.clone());
                #[cfg(feature = "web-api")]
                if let Some(info_key) = _info_key
                    && let Ok(mut map) = INFO_MAP.lock()
                {
                    map.insert(info_key, Some(value.clone().into()));
                }
            }
        }

        merge_option(
            &mut self.real_model.battery.designed_capacity,
            &self.temp_model.battery.designed_capacity,
            Some("bat_capacity_designed"),
        );
        merge_option(
            &mut self.real_model.battery.actually_capacity,
            &self.temp_model.battery.actually_capacity,
            Some("bat_capacity_max"),
        );
        merge_option(
            &mut self.real_model.battery.remain_capacity,
            &self.temp_model.battery.remain_capacity,
            Some("bat_capacity_remain"),
        );
        merge_option(
            &mut self.real_model.battery.voltage,
            &self.temp_model.battery.voltage,
            Some("bat_voltage"),
        );
        merge_option(
            &mut self.real_model.battery.current,
            &self.temp_model.battery.current,
            None,
        );
        merge_option(
            &mut self.real_model.battery.rate,
            &self.temp_model.battery.rate,
            Some("bat_rate"),
        );
        merge_option(
            &mut self.real_model.battery.is_charging,
            &self.temp_model.battery.is_charging,
            Some("bat_state"),
        );

        merge_option(
            &mut self.real_model.cpu.name,
            &self.temp_model.cpu.name,
            Some("cpu_name"),
        );
        merge_option(
            &mut self.real_model.cpu.usage,
            &self.temp_model.cpu.usage,
            Some("cpu_usage"),
        );
        merge_option(
            &mut self.real_model.cpu.package_temperature,
            &self.temp_model.cpu.package_temperature,
            Some("cpu_temperature"),
        );
        merge_option(
            &mut self.real_model.cpu.average_temperature,
            &self.temp_model.cpu.average_temperature,
            None,
        );
        merge_option(
            &mut self.real_model.cpu.power,
            &self.temp_model.cpu.power,
            Some("cpu_power"),
        );
        merge_option(
            &mut self.real_model.cpu.clock_begin_index,
            &self.temp_model.cpu.clock_begin_index,
            Some("cpu_clock_first"),
        );
        merge_option(
            &mut self.real_model.cpu.clock_end_index,
            &self.temp_model.cpu.clock_end_index,
            Some("cpu_clock_last"),
        );
        merge_option(
            &mut self.real_model.cpu.clock,
            &self.temp_model.cpu.clock,
            Some("cpu_clock_avg"),
        );
        merge_option(
            &mut self.real_model.cpu.load,
            &self.temp_model.cpu.load,
            None,
        );
        merge_option(
            &mut self.real_model.cpu.voltage,
            &self.temp_model.cpu.voltage,
            Some("cpu_voltage"),
        );

        merge_option(
            &mut self.real_model.gpu.name,
            &self.temp_model.gpu.name,
            Some("gpu_name"),
        );
        merge_option(
            &mut self.real_model.gpu.temperature,
            &self.temp_model.gpu.temperature,
            Some("gpu_temperature"),
        );
        merge_option(
            &mut self.real_model.gpu.max_temperature,
            &self.temp_model.gpu.max_temperature,
            None,
        );
        merge_option(
            &mut self.real_model.gpu.power,
            &self.temp_model.gpu.power,
            Some("gpu_power"),
        );
        merge_option(
            &mut self.real_model.gpu.speed,
            &self.temp_model.gpu.speed,
            Some("gpu_clock_rms"),
        );
        merge_option(
            &mut self.real_model.gpu.mem_total,
            &self.temp_model.gpu.mem_total,
            None,
        );
        merge_option(
            &mut self.real_model.gpu.mem_free,
            &self.temp_model.gpu.mem_free,
            None,
        );
        merge_option(
            &mut self.real_model.gpu.mem_used,
            &self.temp_model.gpu.mem_used,
            None,
        );
        merge_option(
            &mut self.real_model.gpu.mem_usage,
            &self.temp_model.gpu.mem_usage,
            None,
        );

        merge_option(
            &mut self.real_model.ram.used_size,
            &self.temp_model.ram.used_size,
            Some("mem_used"),
        );
        merge_option(
            &mut self.real_model.ram.free_size,
            &self.temp_model.ram.free_size,
            Some("mem_available"),
        );
        merge_option(
            &mut self.real_model.ram.total_size,
            &self.temp_model.ram.total_size,
            Some("mem_total"),
        );
        merge_option(
            &mut self.real_model.ram.total_swap,
            &self.temp_model.ram.total_swap,
            Some("mem_swap_total"),
        );
        merge_option(
            &mut self.real_model.ram.used_swap,
            &self.temp_model.ram.used_swap,
            Some("mem_swap_used"),
        );
        merge_option(
            &mut self.real_model.ram.free_swap,
            &self.temp_model.ram.free_swap,
            None,
        );

        merge_option(
            &mut self.real_model.fans.fan_speed,
            &self.temp_model.fans.fan_speed,
            None,
        );
        merge_option(
            &mut self.real_model.motherboard.name,
            &self.temp_model.motherboard.name,
            None,
        );

        self.temp_model = Device::default();
    }
}
