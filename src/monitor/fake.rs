use crate::monitor::Updater;
use crate::monitor::hardware_model::Device;
use anyhow::Result;
use std::sync::{Arc, Mutex};

pub struct Fake {}

impl Fake {
    pub fn build() -> Result<Arc<Mutex<Self>>> {
        Ok(Arc::new(Mutex::new(Fake {})))
    }
}

impl Updater for Fake {
    fn update_once(&mut self, device: &mut Device) -> Result<()> {
        device.system.os_name = Some("fake-os".to_string());
        device.system.os_version = Some("0.0".to_string());
        device.system.kernel_version = Some("0.0".to_string());
        device.system.host_name = Some("fake-host".to_string());
        device.system.is_activated = Some(true);

        device.battery.designed_capacity = Some(100.0);
        device.battery.actually_capacity = Some(95.0);
        device.battery.remain_capacity = Some(80.0);
        device.battery.voltage = Some(12.0);
        device.battery.current = Some(1.0);
        device.battery.rate = Some(30.0);
        device.battery.is_charging = Some(false);

        device.cpu.name = Some("fake-cpu".to_string());
        device.cpu.usage = Some(10.0);
        device.cpu.package_temperature = Some(45.0);
        device.cpu.average_temperature = Some(42.0);
        device.cpu.power = Some(20.0);
        device.cpu.clock = Some(3.2);
        device.cpu.load = Some(10.0);
        device.cpu.voltage = Some(1.1);

        device.gpu.name = Some("fake-gpu".to_string());
        device.gpu.power_usage = Some(35.0);
        device.gpu.temperature = Some(50.0);
        device.gpu.clock = Some(1200);
        device.gpu.mem_clock = Some(1600);

        device.ram.total_size = Some(16.0);
        device.ram.used_size = Some(8.0);
        device.ram.free_size = Some(8.0);
        device.ram.total_swap = Some(8.0);
        device.ram.used_swap = Some(1.0);
        device.ram.free_swap = Some(7.0);

        device.fans.cpu_speed = Some(1200);
        device.fans.gpu_speed = Some(1000);
        device.fans.mid_speed = Some(900);

        device.motherboard.name = Some("fake-board".to_string());

        Ok(())
    }

    fn update_slow(&mut self, _device: &mut Device) -> Result<()> {
        Ok(())
    }

    fn update(&mut self, _device: &mut Device) -> Result<()> {
        Ok(())
    }

    fn shutdown(&mut self) -> Result<()> {
        Ok(())
    }
}
