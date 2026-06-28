use crate::util::data_container::DataContainer;

#[allow(unused)]
pub trait QueryField {
    fn query(&self, key: &str) -> QueryResult;
}

/// the option inside Found means whether finding the corresponding field or not
#[allow(unused)]
#[derive(Debug)]
pub enum QueryResult {
    Found(Option<DataContainer>),
    NotFound,
}
