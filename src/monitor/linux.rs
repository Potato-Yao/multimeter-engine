use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use crate::monitor::Updater;
use crate::util::data_container::DataContainer;
use anyhow::Result;

pub struct Linux {

}

#[cfg(target_os = "linux")]
impl Updater for Linux {
    fn update_once(&mut self, map: &mut HashMap<&str, Option<DataContainer>>) -> Result<()> {
        todo!()
    }

    fn update_slow(&mut self, map: &mut HashMap<&str, Option<DataContainer>>) -> Result<()> {
        todo!()
    }

    fn update(&mut self, map: &mut HashMap<&str, Option<DataContainer>>) -> Result<()> {
        todo!()
    }

    fn shutdown(&mut self) -> Result<()> {
        todo!()
    }
}

#[cfg(target_os = "linux")]
impl Linux {
    pub fn build() -> Result<Arc<Mutex<Self>>> {
        todo!()
    }
}

#[cfg(all(target_os = "linux", test))]
mod tests {
    #[test]
    fn test() {
        let sensors = lm_sensors::Initializer::default().initialize().unwrap();

        // Print all chips.
        for chip in sensors.chip_iter(None) {
            if let Some(path) = chip.path() {
                println!("chip: {} at {} ({})", chip, chip.bus(), path.display());
            } else {
                println!("chip: {} at {}", chip, chip.bus());
            }

            // Print all features of the current chip.
            for feature in chip.feature_iter() {
                let name = feature.name().transpose().unwrap().unwrap_or("N/A");
                println!("    {}: {}", name, feature);

                // Print all sub-features of the current chip feature.
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
}
