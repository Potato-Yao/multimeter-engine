use std::collections::HashMap;
use crate::util::data_type_container::DataTypeContainer;
use crate::util::payload::PayLoad;
use crate::web::SERVER_STATE;

pub struct QueryRequest {
    pub target: String,
    pub parameter: Option<InfoMap>,
}

pub type InfoMap = HashMap<String, DataTypeContainer>;

pub fn query_info(query: QueryRequest) -> (PayLoad, SERVER_STATE) {
    todo!()
}
