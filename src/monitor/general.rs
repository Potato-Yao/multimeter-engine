use crate::insert_data;
use crate::monitor::Updater;
use crate::util::data_container::DataContainer;
use anyhow::Result;
use starship_battery::{Battery, Manager, State};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use sysinfo::{Disks, System};

struct SysinfoDisk {}

struct BatteryWrapper(Manager, Battery);

unsafe impl Send for BatteryWrapper {}
unsafe impl Sync for BatteryWrapper {}

struct SysinfoWrapper(System);
unsafe impl Send for SysinfoWrapper {}
unsafe impl Sync for SysinfoWrapper {}

pub struct General {
    battery: BatteryWrapper,
    system: SysinfoWrapper,
}

impl Updater for General {
    fn update_once(&mut self, map: &mut HashMap<&str, Option<DataContainer>>) -> Result<()> {
        Ok(())
    }

    fn update_slow(&mut self, map: &mut HashMap<&str, Option<DataContainer>>) -> Result<()> {
        let disks = Disks::new_with_refreshed_list()
            .iter()
            .map(|e| serde_json::to_string(e).unwrap())
            .collect::<Vec<_>>();
        insert_data!(map, "disk_disk", disks);

        Ok(())
    }

    fn update(&mut self, map: &mut HashMap<&str, Option<DataContainer>>) -> Result<()> {
        self.battery.0.refresh(&mut self.battery.1)?;
        self.system.0.refresh_all();

        let bat = &mut self.battery.1;
        let sys = &self.system.0;
        let cpu = &sys.cpus()[0];
        // map.insert(
        //     "bat_capacity_remain",
        //     Some(DataContainer::Float(joules_to_watt_hours(
        //         bat.energy().value as f64,
        //     ))),
        // );
        insert_data!(
            map,
            "bat_capacity_remain",
            joules_to_watt_hours(bat.energy().value as f64)
        );
        insert_data!(
            map,
            "bat_capacity_designed",
            joules_to_watt_hours(bat.energy_full_design().value as f64)
        );
        insert_data!(
            map,
            "bat_capacity_max",
            joules_to_watt_hours(bat.energy_full().value as f64)
        );
        insert_data!(map, "bat_rate", bat.energy_rate().value as f64);
        insert_data!(map, "bat_voltage", bat.voltage().value as f64);
        insert_data!(
            map,
            "bat_state",
            match bat.state() {
                State::Charging | State::Full => true,
                State::Discharging | State::Unknown | State::Empty => false,
            }
        );
        if let Some(count) = bat.cycle_count() {
            insert_data!(map, "bat_count", count as i32);
        }
        if let Some(val) = System::name() {
            insert_data!(map, "os_name", val);
        }
        if let Some(val) = System::kernel_version() {
            insert_data!(map, "os_kernel_version", val);
        }
        if let Some(val) = System::os_version() {
            insert_data!(map, "os_version", val);
        }
        if let Some(val) = System::host_name() {
            insert_data!(map, "os_host_name", val);
        }
        insert_data!(map, "mem_used", byte_to_gb(sys.used_memory()));
        insert_data!(map, "mem_available", byte_to_gb(sys.available_memory()));
        insert_data!(
            map,
            "mem_percentage",
            sys.used_memory() as f64 / sys.total_memory() as f64
        );
        insert_data!(map, "cpu_name", cpu.name());
        insert_data!(map, "cpu_clock_rms", cpu.frequency());
        insert_data!(map, "cpu_usage", sys.global_cpu_usage() as f64);

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

impl General {
    pub fn build() -> Result<Arc<Mutex<Self>>> {
        let manager = Manager::new()?;
        let bat = manager
            .batteries()?
            .next()
            .transpose()?
            .ok_or_else(|| anyhow::anyhow!("No battery found"))?;
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
