use serde::{Deserialize, Serialize};

pub mod request_executor;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum RequestKind {
    GetInfo,
    ExecuteTool,
}
