use crate::web::model::{PayloadItem, Request, Response, ResponseError};
use log::{debug, error};
use request_executor::execute_request;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

pub mod model;

pub mod request_executor;

pub type ServerState = u32;

pub const SUCCESS_STATE: ServerState = 100;
pub const PARTIAL_SUCCESS_STATE: ServerState = 207;
pub const NOT_FOUND_STATE: ServerState = 404;
const INTERNAL_ERROR_STATE: ServerState = 500;
pub const LATEST_VERSION: u32 = 1;
const DEFAULT_ID: &str = "__default_id__";

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum Command {
    GetInfo,
    ExecuteTool,
    Shutdown,
}

pub fn handle_request(line: String) -> Result<Response, Response> {
    match serde_json::from_str::<Request>(&line) {
        Ok(req) => {
            debug!("Parsed request: {:?}", req);
            execute_request(req)
        }
        Err(_) => Err(parse_error_response(line)),
    }
}

fn parse_error_response(line: String) -> Response {
    let error = format!("Failed to parse request: {:?}", line);
    error!("{}", error);

    let mut payload = BTreeMap::new();
    payload.insert(
        "error".to_string(),
        PayloadItem::Error {
            error: ResponseError {
                code: "invalid_request".to_string(),
                message: error,
            },
        },
    );

    Response {
        version: LATEST_VERSION,
        id: DEFAULT_ID.to_string(),
        state: NOT_FOUND_STATE,
        payload,
    }
}
