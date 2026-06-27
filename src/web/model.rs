use crate::util::data_container::DataContainer;
use crate::util::info_map::InfoMap;
use crate::util::payload::PayLoad;
use crate::web::RequestKind;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::collections::{BTreeMap, HashMap};

#[derive(Serialize, Deserialize, Debug)]
pub struct Request {
    pub version: u32,
    pub id: String,
    pub kind: RequestKind,
    pub payload: PayLoad,
}

#[derive(Serialize, Debug)]
pub struct Response {
    pub version: u32,
    pub id: String,
    pub state: u32,
    pub payload: PayLoad,
}

#[derive(Serialize, Debug)]
#[serde(untagged)]
pub enum WebResponse {
    V1(Response),
    V2(V2Response),
}

#[derive(Serialize, Deserialize, Debug)]
pub struct V2Request {
    pub version: u32,
    pub id: String,
    pub command: V2Command,
    pub payload: V2RequestPayload,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum V2Command {
    GetInfo,
    ExecuteTool,
    Shutdown,
}

pub type V2RequestPayload = BTreeMap<String, Option<Map<String, Value>>>;

#[derive(Serialize, Debug)]
pub struct V2Response {
    pub version: u32,
    pub id: String,
    pub state: u32,
    pub payload: BTreeMap<String, V2PayloadItem>,
}

#[derive(Serialize, Debug)]
#[serde(untagged)]
pub enum V2PayloadItem {
    Success {
        result: DataContainer,
        #[serde(flatten, skip_serializing_if = "HashMap::is_empty")]
        addition: InfoMap,
    },
    Error {
        error: V2Error,
    },
}

#[derive(Serialize, Debug)]
pub struct V2Error {
    pub code: String,
    pub message: String,
}
