use std::collections::{HashMap, HashSet};

pub type IncludeQuery = HashSet<String>;
pub type FieldsQuery = HashMap<String, HashSet<String>>;
pub type SortQuery = Vec<String>;

#[derive(Debug)]
pub struct Query {
    pub include: IncludeQuery,
    pub fields: FieldsQuery,
    pub sort: SortQuery,
    pub page: Option<PageQuery>,
    pub filter: Option<FilterQuery>,
}

#[derive(Debug)]
pub enum PageQuery {
    OffsetBased { offset: u32, limit: u32 },
    PageBased { number: u32, size: u32 },
    CursorBased(String),
}

#[derive(Debug)]
pub enum FilterQuery {
    Rsql(rsql_rs::ast::expr::Expr),
}
