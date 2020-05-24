use crate::model::link::WrappedUri;
/// Pagination links
#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
pub struct Pagination {
    pub first: Option<WrappedUri>,
    pub prev: Option<WrappedUri>,
    pub next: Option<WrappedUri>,
    pub last: Option<WrappedUri>,
}
