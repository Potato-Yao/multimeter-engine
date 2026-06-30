use crate::monitor::Updater;
use crate::monitor::model::Device;
use anyhow::Result;
use starship_battery::{Battery, Manager, State};
use std::sync::{Arc, Mutex};
use sysinfo::System;

struct BatteryWrapper(Manager, Option<Battery>);

unsafe impl Send for BatteryWrapper {}
unsafe impl Sync for BatteryWrapper {}

struct SysinfoWrapper(System);
unsafe impl Send for SysinfoWrapper {}
unsafe impl Sync for SysinfoWrapper {}

pub struct CrossPlatform {
    battery: BatteryWrapper,
    system: SysinfoWrapper,
}

impl Updater for CrossPlatform {
    fn update_once(&mut self, device: &mut Device) -> Result<()> {
        self.system.0.refresh_memory();
        self.system.0.refresh_cpu_all();

        let sys = &self.system.0;

        device.ram.total_size = Some(byte_to_gb(sys.total_memory()));
        device.ram.total_swap = Some(byte_to_gb(sys.total_swap()));
        device.system.os_name = System::name();
        device.system.kernel_version = System::kernel_version();
        device.system.os_version = System::os_version();
        device.system.host_name = System::host_name();

        if let Some(cpu) = sys.cpus().first() {
            device.cpu.name = Some(cpu.name().to_string());
        }

        if let Some(bat) = self.battery.1.as_mut() {
            self.battery.0.refresh(bat)?;
            device.battery.designed_capacity =
                Some(joules_to_watt_hours(bat.energy_full_design().value as f64));
            device.battery.actually_capacity =
                Some(joules_to_watt_hours(bat.energy_full().value as f64));
        }

        Ok(())
    }

    fn update_slow(&mut self, _device: &mut Device) -> Result<()> {
        Ok(())
    }

    fn update(&mut self, device: &mut Device) -> Result<()> {
        self.system.0.refresh_memory();
        self.system.0.refresh_cpu_all();

        let sys = &self.system.0;

        device.ram.used_size = Some(byte_to_gb(sys.used_memory()));
        device.ram.used_swap = Some(byte_to_gb(sys.used_swap()));
        device.ram.free_size = Some(byte_to_gb(sys.available_memory()));
        device.ram.free_swap = Some(byte_to_gb(sys.total_swap().saturating_sub(sys.used_swap())));

        if let Some(cpu) = sys.cpus().first() {
            device.cpu.clock = Some(cpu.frequency() as f64);
        }
        device.cpu.usage = Some(sys.global_cpu_usage() as f64);

        if let Some(bat) = self.battery.1.as_mut() {
            self.battery.0.refresh(bat)?;
            device.battery.remain_capacity = Some(joules_to_watt_hours(bat.energy().value as f64));
            device.battery.rate = Some(bat.energy_rate().value as f64);
            device.battery.voltage = Some(bat.voltage().value as f64);
            device.battery.is_charging = Some(matches!(
                bat.state(),
                State::Charging | State::Full | State::LimitedFull
            ));
        }

        Ok(())
    }

    fn shutdown(&mut self) -> Result<()> {
        Ok(())
    }
}

fn joules_to_watt_hours(value: f64) -> f64 {
    value * 0.000278
}

fn byte_to_gb(value: u64) -> f64 {
    (value as f64) / 1024.0 / 1024.0 / 1024.0
}

impl CrossPlatform {
    pub fn build() -> Result<Arc<Mutex<Self>>> {
        let manager = Manager::new()?;
        let bat = manager.batteries()?.next().transpose()?;
        let sysinfo = System::new_all();

        Ok(Arc::new(Mutex::new(Self {
            battery: BatteryWrapper(manager, bat),
            system: SysinfoWrapper(sysinfo),
        })))
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_sysinfo() {
        use sysinfo::{Components, Disks, Networks, System};

        // Please note that we use "new_all" to ensure that all lists of
        // CPUs and processes are filled!
        let mut sys = System::new_all();

        // First we update all information of our `System` struct.
        sys.refresh_all();

        println!("=> system:");
        // RAM and swap information:
        println!("total memory: {} bytes", sys.total_memory());
        println!("used memory : {} bytes", sys.used_memory());
        println!("total swap  : {} bytes", sys.total_swap());
        println!("used swap   : {} bytes", sys.used_swap());

        // Display system information:
        println!("System name:             {:?}", System::name());
        println!("System kernel version:   {:?}", System::kernel_version());
        println!("System OS version:       {:?}", System::os_version());
        println!("System host name:        {:?}", System::host_name());

        // Number of CPUs:
        println!("NB CPUs: {}", sys.cpus().len());

        // Display processes ID, name and disk usage:
        for (pid, process) in sys.processes() {
            println!("[{pid}] {:?} {:?}", process.name(), process.disk_usage());
        }

        // We display all disks' information:
        println!("=> disks:");
        let disks = Disks::new_with_refreshed_list();
        for disk in &disks {
            println!("{disk:?}");
        }

        // Network interfaces name, total data received and total data transmitted:
        let networks = Networks::new_with_refreshed_list();
        println!("=> networks:");
        for (interface_name, data) in &networks {
            println!(
                "{interface_name}: {} B (down) / {} B (up)",
                data.total_received(),
                data.total_transmitted(),
            );
            // If you want the amount of data received/transmitted since last call
            // to `Networks::refresh`, use `received`/`transmitted`.
        }

        // Components temperature:
        let components = Components::new_with_refreshed_list();
        println!("=> components:");
        for component in &components {
            println!("{component:?}");
        }
    }
}
