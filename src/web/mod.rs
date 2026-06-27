use crate::util::data_container::DataContainer;
use crate::util::payload::PayLoad;
use crate::web::model::{Request, Response, V2Request, WebResponse};
use log::{debug, error, info};
use request_executor::execute_request;
use serde::{Deserialize, Serialize};

pub mod model;

pub mod request_executor;

pub type ServerState = u32;

pub const SUCCESS_STATE: ServerState = 100;
pub const PARTIAL_SUCCESS_STATE: ServerState = 207;
pub const NOT_FOUND_STATE: ServerState = 404;
const INTERNAL_ERROR_STATE: ServerState = 500;
pub const LATEST_VERSION: u32 = 2;
const DEFAULT_ID: &str = "__default_id__";

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
pub enum RequestKind {
    GetInfo,
    ExecuteTool,
    Shutdown,
}

pub fn handle_request(line: String) -> Result<WebResponse, WebResponse> {
    let parsed = match serde_json::from_str::<serde_json::Value>(&line) {
        Ok(value) => value,
        Err(_) => return Err(WebResponse::V1(parse_error_response(line))),
    };

    match parsed.get("version").and_then(|v| v.as_u64()) {
        Some(1) => match serde_json::from_value::<Request>(parsed) {
            Ok(req) => {
                info!("Request with id {} parsed.", req.id);
                debug!("Parsed request: {:?}", req);
                execute_request(req)
                    .map(WebResponse::V1)
                    .map_err(WebResponse::V1)
            }
            Err(_) => Err(WebResponse::V1(parse_error_response(line))),
        },
        Some(2) => match serde_json::from_value::<V2Request>(parsed) {
            Ok(req) => {
                info!("V2 request with id {} parsed.", req.id);
                debug!("Parsed v2 request: {:?}", req);
                request_executor::execute_v2_request(req)
                    .map(WebResponse::V2)
                    .map_err(WebResponse::V2)
            }
            Err(_) => Err(WebResponse::V1(parse_error_response(line))),
        },
        _ => Err(WebResponse::V1(parse_error_response(line))),
    }
}

fn parse_error_response(line: String) -> Response {
    let error = format!("Failed to parse request: {:?}", line);
    error!("{}", error);

    Response {
        version: 1,
        id: DEFAULT_ID.to_string(),
        state: NOT_FOUND_STATE,
        payload: PayLoad {
            value: DataContainer::from(error),
            addition: None,
        },
    }
}
