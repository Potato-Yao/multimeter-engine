use crate::util::data_container::DataContainer;

pub trait QueryField {
    fn query(&self, key: &str) -> QueryResult;
}

/// the option inside Found means whether finding the corresponding field or not
#[derive(Debug)]
pub enum QueryResult {
    Found(Option<DataContainer>),
    NotFound,
}
