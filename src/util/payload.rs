use crate::util::data_container::DataContainer;
use crate::util::info_map::InfoMap;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct PayLoad {
    pub value: DataContainer,
    pub addition: Option<InfoMap>,
}
