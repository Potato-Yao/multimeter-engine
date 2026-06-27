use crate::monitor::{QueryRequest, query_info};
use crate::util::data_container::DataContainer;
use crate::util::info_map::InfoMap;
use crate::util::payload::PayLoad;
use crate::web::model::{
    Request, Response, V2Command, V2Error, V2PayloadItem, V2Request, V2Response,
};
use crate::web::{
    INTERNAL_ERROR_STATE, LATEST_VERSION, NOT_FOUND_STATE, PARTIAL_SUCCESS_STATE, RequestKind,
    SUCCESS_STATE,
};
use log::debug;
use serde_json::{Map, Value};
use std::collections::BTreeMap;

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
        }
        RequestKind::Shutdown => {
            // shutdown()
            todo!()
        }
    }
    .map_err(|e| Response {
        version: req.version,
        id: req.id.clone(),
        state: INTERNAL_ERROR_STATE,
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

pub fn execute_v2_request(req: V2Request) -> Result<V2Response, V2Response> {
    debug!("V2 request is executing: {:?}", req);

    match req.command {
        V2Command::GetInfo => handle_v2_get_info(req),
        V2Command::ExecuteTool => Err(v2_request_error(
            req.id,
            "not_implemented",
            "execute_tool is not implemented",
        )),
        V2Command::Shutdown => Err(v2_request_error(
            req.id,
            "not_implemented",
            "shutdown is not implemented",
        )),
    }
}

fn handle_v2_get_info(req: V2Request) -> Result<V2Response, V2Response> {
    let mut payload = BTreeMap::new();
    let mut failures = 0;

    for (target, params) in req.payload {
        let parameter = match params.map(json_map_to_info_map).transpose() {
            Ok(parameter) => parameter,
            Err(message) => {
                failures += 1;
                payload.insert(
                    target,
                    V2PayloadItem::Error {
                        error: V2Error {
                            code: "invalid_parameter".to_string(),
                            message,
                        },
                    },
                );
                continue;
            }
        };

        match query_info(QueryRequest {
            target: target.clone(),
            parameter,
        }) {
            Ok(result) => {
                payload.insert(
                    target,
                    V2PayloadItem::Success {
                        result: result.value,
                        addition: result.addition.unwrap_or_default(),
                    },
                );
            }
            Err(e) => {
                failures += 1;
                payload.insert(
                    target,
                    V2PayloadItem::Error {
                        error: V2Error {
                            code: "query_failed".to_string(),
                            message: e.to_string(),
                        },
                    },
                );
            }
        }
    }

    let state = if failures == 0 {
        SUCCESS_STATE
    } else if failures == payload.len() {
        INTERNAL_ERROR_STATE
    } else {
        PARTIAL_SUCCESS_STATE
    };

    let response = V2Response {
        version: req.version,
        id: req.id,
        state,
        payload,
    };

    if state == SUCCESS_STATE || state == PARTIAL_SUCCESS_STATE {
        Ok(response)
    } else {
        Err(response)
    }
}

fn v2_request_error(id: String, code: &str, message: &str) -> V2Response {
    let mut payload = BTreeMap::new();
    payload.insert(
        "error".to_string(),
        V2PayloadItem::Error {
            error: V2Error {
                code: code.to_string(),
                message: message.to_string(),
            },
        },
    );

    V2Response {
        version: 2,
        id,
        state: NOT_FOUND_STATE,
        payload,
    }
}

fn json_map_to_info_map(map: Map<String, Value>) -> Result<InfoMap, String> {
    map.into_iter()
        .map(|(key, value)| json_value_to_data_container(value).map(|value| (key, value)))
        .collect()
}

fn json_value_to_data_container(value: Value) -> Result<DataContainer, String> {
    match value {
        Value::Null => Ok(DataContainer::Null),
        Value::Bool(v) => Ok(DataContainer::Boolean(v)),
        Value::Number(v) => {
            if let Some(v) = v.as_i64() {
                i32::try_from(v)
                    .map(DataContainer::Int)
                    .map_err(|_| format!("integer parameter out of range: {v}"))
            } else if let Some(v) = v.as_u64() {
                Ok(DataContainer::UnsignedLong(v))
            } else if let Some(v) = v.as_f64() {
                Ok(DataContainer::Float(v))
            } else {
                Err(format!("unsupported number parameter: {v}"))
            }
        }
        Value::String(v) => Ok(DataContainer::Text(v)),
        Value::Array(v) => v
            .into_iter()
            .map(json_value_to_data_container)
            .collect::<Result<Vec<_>, _>>()
            .map(DataContainer::Array),
        Value::Object(v) => v
            .into_iter()
            .map(|(key, value)| json_value_to_data_container(value).map(|value| (key, value)))
            .collect::<Result<_, _>>()
            .map(DataContainer::Object),
    }
}
