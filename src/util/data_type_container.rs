use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
#[serde(untagged)]
pub enum DataTypeContainer {
    Int(i32),
    Float(f64),
    Text(String),
    Boolean(bool),
}
