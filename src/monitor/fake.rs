use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use crate::monitor::Updater;
use crate::util::data_container::DataContainer;
use anyhow::{Result};

pub struct Fake {
}

impl Fake {
    pub fn build() -> Result<Arc<Mutex<Self>>> {
        Ok(Arc::new(Mutex::new(Fake {})))
    }
}

impl Updater for Fake {
    fn update_once(&mut self, map: &mut HashMap<&str, Option<DataContainer>>) -> anyhow::Result<()> {
        for (_k, v) in map.iter_mut() {
            *v = Some(DataContainer::Int(0));
        }

        Ok(())
    }

    fn update_slow(&mut self, map: &mut HashMap<&str, Option<DataContainer>>) -> anyhow::Result<()> {
        Ok(())
    }

    fn update(&mut self, map: &mut HashMap<&str, Option<DataContainer>>) -> anyhow::Result<()> {
        Ok(())
    }

    fn shutdown(&mut self) -> anyhow::Result<()> {
        todo!()
    }
}
