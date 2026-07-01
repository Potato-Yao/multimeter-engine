use crate::util::data_container::DataContainer;
use crate::util::info_map::InfoMap;

#[allow(unused)]
pub trait QueryField {
    fn query(&self, key: &str, attach: Option<&InfoMap>) -> QueryResult;
}

/// the option inside Found means whether finding the corresponding field or not
#[allow(unused)]
#[derive(Debug)]
pub enum QueryResult {
    Found(Option<DataContainer>),
    NotFound,
}
