use crate::monitor::InfoMap;
use serde::{Deserialize, Serialize};
use crate::util::data_container::DataContainer;

#[derive(Serialize, Deserialize, Debug)]
pub struct PayLoad {
    pub value: DataContainer,
    pub addition: Option<InfoMap>,
}
