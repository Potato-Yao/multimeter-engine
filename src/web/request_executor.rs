use crate::monitor::{QueryRequest, query_info};
use crate::util::data_container::DataContainer;
use crate::util::info_map::InfoMap;
use crate::web::model::{PayloadItem, Request, Response, ResponseError};
use crate::web::{
    Command, INTERNAL_ERROR_STATE, LATEST_VERSION, NOT_FOUND_STATE, PARTIAL_SUCCESS_STATE,
    SUCCESS_STATE,
};
use log::debug;
use serde_json::{Map, Value};
use std::collections::BTreeMap;

pub fn execute_request(req: Request) -> Result<Response, Response> {
    debug!("Request is executing: {:?}", req);

    match req.version {
        1 => handle_command(req),
        _ => Err(Response {
            version: LATEST_VERSION,
            id: req.id,
            state: NOT_FOUND_STATE,
            payload: error_payload(
                "unknown_version",
                format!("Unsupported request version: {}", req.version),
            ),
        }),
    }
}

fn handle_command(req: Request) -> Result<Response, Response> {
    match req.command {
        Command::GetInfo => handle_get_info(req),
        Command::ExecuteTool => Err(request_error(
            req.id,
            "not_implemented",
            "execute_tool is not implemented",
        )),
        Command::Shutdown => Err(request_error(
            req.id,
            "not_implemented",
            "shutdown is not implemented",
        )),
    }
}

fn handle_get_info(req: Request) -> Result<Response, Response> {
    let mut payload = BTreeMap::new();
    let mut failures = 0;

    for (target, params) in req.payload {
        let parameter = match params.map(json_map_to_info_map).transpose() {
            Ok(parameter) => parameter,
            Err(message) => {
                failures += 1;
                payload.insert(
                    target,
                    PayloadItem::Error {
                        error: ResponseError {
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
                    PayloadItem::Success {
                        result: result.value,
                        addition: result.addition.unwrap_or_default(),
                    },
                );
            }
            Err(e) => {
                failures += 1;
                payload.insert(
                    target,
                    PayloadItem::Error {
                        error: ResponseError {
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

    let response = Response {
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

fn request_error(id: String, code: &str, message: &str) -> Response {
    Response {
        version: LATEST_VERSION,
        id,
        state: NOT_FOUND_STATE,
        payload: error_payload(code, message),
    }
}

fn error_payload(code: &str, message: impl Into<String>) -> BTreeMap<String, PayloadItem> {
    let mut payload = BTreeMap::new();
    payload.insert(
        "error".to_string(),
        PayloadItem::Error {
            error: ResponseError {
                code: code.to_string(),
                message: message.into(),
            },
        },
    );
    payload
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
