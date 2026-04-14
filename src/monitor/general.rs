use crate::monitor::Updater;
use crate::util::data_container::DataContainer;
use anyhow::Result;
use starship_battery::{Battery, Manager, State};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use crate::insert_data;

struct BatteryWrapper(Manager, Battery);

unsafe impl Send for BatteryWrapper {}
unsafe impl Sync for BatteryWrapper {}

pub struct General {
    battery: BatteryWrapper,
}

impl Updater for General {
    fn update_once(&mut self, map: &mut HashMap<&str, Option<DataContainer>>) -> Result<()> {
        Ok(())
    }

    fn update_slow(&mut self, map: &mut HashMap<&str, Option<DataContainer>>) -> Result<()> {
        Ok(())
    }

    fn update(&mut self, map: &mut HashMap<&str, Option<DataContainer>>) -> Result<()> {
        self.battery.0.refresh(&mut self.battery.1)?;

        let bat = &mut self.battery.1;
        // map.insert(
        //     "bat_capacity_remain",
        //     Some(DataContainer::Float(joules_to_watt_hours(
        //         bat.energy().value as f64,
        //     ))),
        // );
        insert_data!(map, "bat_capacity_remain", joules_to_watt_hours(bat.energy().value as f64));
        insert_data!(map, "bat_capacity_designed", joules_to_watt_hours(bat.energy_full_design().value as f64));
        insert_data!(map, "bat_capacity_max", joules_to_watt_hours(bat.energy_full().value as f64));
        insert_data!(map, "bat_rate", bat.energy_rate().value as f64);
        insert_data!(map, "bat_voltage", bat.voltage().value as f64);
        insert_data!(map, "bat_state", match bat.state() {
            State::Charging | State::Full => true,
            State::Discharging | State::Unknown | State::Empty => false,
        });
        if let Some(count) = bat.cycle_count() {
            insert_data!(map, "bat_count", count as i32);
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

impl General {
    pub fn build() -> Result<Arc<Mutex<Self>>> {
        let manager = Manager::new()?;
        let bat = manager
            .batteries()?
            .next()
            .transpose()?
            .ok_or_else(|| anyhow::anyhow!("No battery found"))?;

        Ok(Arc::new(Mutex::new(Self {
            battery: BatteryWrapper(manager, bat),
        })))
    }
}
