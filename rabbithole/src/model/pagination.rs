use crate::model::link::RawUri;

/// Pagination links
#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
pub struct Pagination {
    pub first: Option<RawUri>,
    pub prev: Option<RawUri>,
    pub next: Option<RawUri>,
    pub last: Option<RawUri>,
}
