use serde::{Deserialize, Serialize};
use crate::core::RequestKind;
use crate::util::payload::PayLoad;

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
