#[cfg(any(feature = "web-api", feature = "native-api"))]
use crate::util::data_container::DataContainer;
#[cfg(any(feature = "web-api", feature = "native-api"))]
use crate::util::payload::PayLoad;
#[cfg(any(feature = "web-api", feature = "native-api"))]
use crate::web::model::{Request, Response};
#[cfg(any(feature = "web-api", feature = "native-api"))]
use log::{debug, error, info};
#[cfg(any(feature = "web-api", feature = "native-api"))]
use request_executor::execute_request;
#[cfg(any(feature = "web-api", feature = "native-api"))]
use serde::{Deserialize, Serialize};

#[cfg(any(feature = "web-api", feature = "native-api"))]
pub mod model;

#[cfg(any(feature = "web-api", feature = "native-api"))]
pub mod request_executor;

#[cfg(any(feature = "web-api", feature = "native-api"))]
pub type ServerState = u32;

#[cfg(any(feature = "web-api", feature = "native-api"))]
pub const SUCCESS_STATE: ServerState = 100;
#[cfg(any(feature = "web-api", feature = "native-api"))]
pub const NOT_FOUND_STATE: ServerState = 404;
#[cfg(any(feature = "web-api", feature = "native-api"))]
const INTERNAL_ERROR_STATE: ServerState = 500;
#[cfg(any(feature = "web-api", feature = "native-api"))]
pub const LATEST_VERSION: u32 = 1;
#[cfg(any(feature = "web-api", feature = "native-api"))]
const DEFAULT_ID: &str = "__default_id__";

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
#[cfg(any(feature = "web-api", feature = "native-api"))]
pub enum RequestKind {
    GetInfo,
    ExecuteTool,
    Shutdown,
}

#[cfg(any(feature = "web-api", feature = "native-api"))]
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
