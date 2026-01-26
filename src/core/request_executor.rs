use log::debug;
use crate::core::RequestKind;
use crate::monitor::{query_info, QueryRequest};
use crate::util::data_container::DataContainer;
use crate::util::payload::PayLoad;
use crate::web::{LATEST_VERSION, NOT_FOUND_STATE, SUCCESS_STATE};
use crate::web::model::{Request, Response};

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
    let payload = PayLoad {
        value: DataContainer::from(""),
        addition: None,
    };
    let state = SUCCESS_STATE;

    let info = match req.kind {
        RequestKind::GetInfo => {
            query_info(QueryRequest {
                target: String::from(req.payload.value),
                parameter: req.payload.addition,
            })
        }
        RequestKind::ExecuteTool => {
            todo!()
        }
    };

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
