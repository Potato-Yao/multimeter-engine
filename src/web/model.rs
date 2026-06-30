use crate::util::data_container::DataContainer;
use crate::util::info_map::InfoMap;
use crate::web::Command;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::collections::{BTreeMap, HashMap};

#[derive(Serialize, Deserialize, Debug)]
pub struct Request {
    pub version: u32,
    pub id: String,
    pub command: Command,
    pub payload: RequestPayload,
}

pub type RequestPayload = BTreeMap<String, Option<Map<String, Value>>>;

#[derive(Serialize, Debug)]
pub struct Response {
    pub version: u32,
    pub id: String,
    pub state: u32,
    pub payload: BTreeMap<String, PayloadItem>,
}

#[derive(Serialize, Debug)]
#[serde(untagged)]
pub enum PayloadItem {
    Success {
        result: DataContainer,
        #[serde(flatten, skip_serializing_if = "HashMap::is_empty")]
        addition: InfoMap,
    },
    Error {
        error: ResponseError,
    },
}

#[derive(Serialize, Debug)]
pub struct ResponseError {
    pub code: String,
    pub message: String,
}
