use crate::web::model::{Request, Response};
use request_executor::execute_request;
use log::{debug, error, info};
use serde::{Deserialize, Serialize};
use crate::util::data_container::DataContainer;
use crate::util::payload::PayLoad;

pub mod model;
pub mod request_executor;

pub type SERVER_STATE = u32;

pub const SUCCESS_STATE: SERVER_STATE = 100;
pub const NOT_FOUND_STATE: SERVER_STATE = 404;
const INTERNAL_ERROR_STATE: SERVER_STATE = 500;
pub const LATEST_VERSION: u32 = 1;
const DEFAULT_ID: &str = "__default_id__";

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum RequestKind {
    GetInfo,
    ExecuteTool,
}

pub fn handle_request(line: String) -> Result<Response, Response> {
    if let Ok(req) = serde_json::from_str::<Request>(&line) {
        info!("Request with id {} parsed.", req.id);
        debug!("Parsed request: {:?}", req);

        execute_request(req)
    } else {
        let error = format!("Failed to parse request: {:?}", line);
        error!("{}", error);

        Err(Response {
            version: 1,
            id: DEFAULT_ID.to_string(),
            state: NOT_FOUND_STATE,
            payload: PayLoad {
                value: DataContainer::from(error),
                addition: None,
            },
        })
    }
}
