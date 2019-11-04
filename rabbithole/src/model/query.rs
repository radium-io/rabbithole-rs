use crate::RbhResult;
use http::Uri;
use regex::Regex;
use std::collections::{HashMap, HashSet};
use std::iter::FromIterator;
use url::Url;

pub type IncludeQuery = HashSet<String>;
pub type FieldsQuery = HashMap<String, HashSet<String>>;
pub type SortQuery = Vec<String>;

#[derive(Debug, Default)]
pub struct Query {
    /// When include is:
    ///   1. `None`: all included fields will be added
    ///   2. `Some(<empty-query>)`: no included field will be added
    ///   3. `Some(<some-query>)`:
    ///     1. No query matches the existing fields: like branch 2
    ///     2. Otherwise: all matched fields will be added
    pub include: Option<IncludeQuery>,
    /// If `ty` is marked in `fields` map, then all resources of this type will remove all the `attributes` and `relationships`
    /// not in `fields[<ty>]`
    pub fields: FieldsQuery,
    /// When sort is:
    ///   1. empty: no sorting at all, clients should not expect the order of the result
    ///   2. some values, but none of the values matches: like branch 1
    ///   3. some values, and some of the items matches: sorting result with the order of the matched sort-query item
    pub sort: SortQuery,
    pub page: Option<PageQuery>,
    pub filter: Option<FilterQuery>,
}

lazy_static! {
    static ref KEY_REGEX: Regex = Regex::new(r#"(?P<name>\w+)\[(?P<param>[\w-_]+)\]"#).unwrap();
}

impl Query {
    pub fn from_url(url: Url, page_type: &str, filter_type: &str) -> RbhResult<Query> {
        let mut include_query: IncludeQuery = Default::default();
        let mut include_query_exist = false;
        let mut sort_query: SortQuery = Default::default();
        let mut filter_map: HashMap<String, String> = Default::default();
        let mut fields_map: HashMap<String, HashSet<String>> = Default::default();

        for (key, value) in url.query_pairs() {
            if key == "include" {
                include_query_exist = true;

                for v in value.split(",").filter(|s: &str| !s.is_empty()) {
                    include_query.insert(v);
                }
                continue;
            }

            if key == "sort" {
                for v in value.split(",").filter(|s: &str| !s.is_empty()) {
                    sort_query.push(v);
                }
                continue;
            }

            if let Some(cap) = (&KEY_REGEX as &Regex).captures(key.as_ref()) {
                if let Some(res) = (cap.name("name"), cap.name("param")) {
                    let values: HashSet<String> =
                        value.split(",").filter(|s: &str| !s.is_empty()).collect();

                    if let Some(origin_fields) = fields_map.get_mut(res.as_str()) {
                        for v in values {
                            origin_fields.insert(v);
                        }
                    } else {
                        fields_map.insert(res.into(), values);
                    }
                }
                continue;
            }

            if let Some()
        }

        let include = if include_query_exist { Some(include_query) } else { None };
        let fields = if fields_query_exist { Some(fields_query) } else { None };
        let sort = sort_query;
        let page = if page_query_exist { Some(page_query) } else { None };
        let filter = if filter_query_exist { Some(filter_query) } else { None };

        Ok(Query { include, fields, sort, page, filter })
    }
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
