use serde::{Deserialize, Serialize};
use crate::monitor::InfoMap;

#[derive(Serialize, Deserialize, Debug)]
pub struct PayLoad {
    pub value: String,
    pub addition: Option<InfoMap>,
}