/// Pagination links
#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
pub struct Pagination {
    pub first: Option<String>,
    pub prev: Option<String>,
    pub next: Option<String>,
    pub last: Option<String>,
}
