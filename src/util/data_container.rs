use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone)]
#[serde(untagged)]
pub enum DataContainer {
    Int(i32),
    UnsignedLong(u64),
    Float(f64),
    Text(String),
    Boolean(bool),
    Array(Vec<DataContainer>),
}

impl From<i32> for DataContainer {
    fn from(value: i32) -> Self {
        DataContainer::Int(value)
    }
}

impl From<u64> for DataContainer {
    fn from(value: u64) -> Self {
        DataContainer::UnsignedLong(value)
    }
}

impl From<f64> for DataContainer {
    fn from(value: f64) -> Self {
        DataContainer::Float(value)
    }
}

impl From<String> for DataContainer {
    fn from(value: String) -> Self {
        DataContainer::Text(value)
    }
}

impl From<&str> for DataContainer {
    fn from(value: &str) -> Self {
        DataContainer::Text(value.to_string())
    }
}

impl From<bool> for DataContainer {
    fn from(value: bool) -> Self {
        DataContainer::Boolean(value)
    }
}

impl From<Vec<DataContainer>> for DataContainer {
    fn from(value: Vec<DataContainer>) -> Self {
        DataContainer::Array(value)
    }
}

impl From<Vec<String>> for DataContainer {
    fn from(value: Vec<String>) -> Self {
        DataContainer::Array(value.into_iter().map(DataContainer::from).collect())
    }
}

impl From<DataContainer> for String {
    fn from(value: DataContainer) -> Self {
        match value {
            DataContainer::Int(v) => v.to_string(),
            DataContainer::UnsignedLong(v) => v.to_string(),
            DataContainer::Float(v) => v.to_string(),
            DataContainer::Text(v) => v,
            DataContainer::Boolean(v) => v.to_string(),
            DataContainer::Array(v) => v
                .into_iter()
                .map(|item| String::from(item))
                .collect::<Vec<String>>()
                .join(", "),
        }
    }
}
