use crate::monitor::{QueryRequest, query_info};
use crate::util::payload::PayLoad;
use crate::web::RequestKind;
use crate::web::model::{Request, Response};
use crate::web::{LATEST_VERSION, NOT_FOUND_STATE, SUCCESS_STATE};
use log::debug;
use crate::shutdown;

pub fn execute_request(req: Request) -> Result<Response, Response> {
    debug!("Request is executing: {:?}", req);

    match req.version {
        1 => handle_v1_request(req),
        _ => Err(Response {
            version: LATEST_VERSION,
            id: req.id,
            state: NOT_FOUND_STATE,
            payload: PayLoad {
                value: format!("Unknown request version: {}", req.version).into(),
                addition: None,
            },
        }),
    }
}

fn handle_v1_request(req: Request) -> Result<Response, Response> {
    let state = SUCCESS_STATE;

    let payload = match req.kind {
        RequestKind::GetInfo => query_info(QueryRequest {
            target: String::from(req.payload.value),
            parameter: req.payload.addition,
        }),
        RequestKind::ExecuteTool => {
            todo!()
        },
        RequestKind::Shutdown => {
            shutdown()
        },
    }
    .map_err(|e| Response {
        version: req.version,
        id: req.id.clone(),
        state: NOT_FOUND_STATE,
        payload: PayLoad {
            value: format!("Failed to process request: {}", e).into(),
            addition: None,
        },
    })?;

    let res = Response {
        version: req.version,
        id: req.id,
        state,
        payload,
    };

    if state == SUCCESS_STATE {
        Ok(res)
    } else {
        Err(res)
    }
}
