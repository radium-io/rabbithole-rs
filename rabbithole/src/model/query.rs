use crate::error::RabbitholeError;

use crate::RbhResult;

use regex::Regex;
use rsql_rs::ast::expr::Expr;
use rsql_rs::parser::rsql::RsqlParser;
use rsql_rs::parser::Parser;
use std::collections::{HashMap, HashSet};

use std::str::FromStr;
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
    /// If `ty` is marked in `fields` map:
    ///   1. If `fields[<ty>]` is not empty: all resources of this type will remove the `attributes` and `relationships` which are not in `fields[<ty>]`
    ///   2. If `fields[<ty>]` is empty: all the fields of the resources with this type will be removed
    /// Else: retain all fields
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
        let mut fields_map: FieldsQuery = Default::default();
        let mut page_map: HashMap<String, String> = Default::default();

        for (key, value) in url.query_pairs() {
            if key == "include" {
                include_query_exist = true;

                for v in value.split(",").filter(|s| !s.is_empty()) {
                    include_query.insert(v.to_string());
                }
                continue;
            }

            if key == "sort" {
                for v in value.split(",").filter(|s| !s.is_empty()) {
                    sort_query.push(v.to_string());
                }
                continue;
            }

            if let Some(cap) = (&KEY_REGEX as &Regex).captures(key.as_ref()) {
                if let (Some(name), Some(param)) = (cap.name("name"), cap.name("param")) {
                    let name = name.as_str();
                    let param = param.as_str();

                    if name == "fields" {
                        let values: HashSet<String> = value
                            .split(",")
                            .filter(|s| !s.is_empty())
                            .map(ToString::to_string)
                            .collect();
                        if let Some(origin_fields) = fields_map.get_mut(param) {
                            for v in values {
                                origin_fields.insert(v);
                            }
                        } else {
                            fields_map.insert(param.into(), values);
                        }
                    } else if name == "filter" && !value.is_empty() {
                        filter_map.insert(param.into(), value.to_string());
                    } else if name == "page" {
                        page_map.insert(param.into(), value.to_string());
                    }
                }
            }
        }

        let include = if include_query_exist { Some(include_query) } else { None };
        let sort = sort_query;
        let page = PageQuery::new(page_type, &page_map)?;
        let filter = FilterQuery::new(filter_type, filter_map)?;

        Ok(Query { include, fields: fields_map, sort, page, filter })
    }
}

#[derive(Debug)]
pub enum PageQuery {
    OffsetBased { offset: u32, limit: u32 },
    PageBased { number: u32, size: u32 },
    CursorBased(String),
}

impl PageQuery {
    pub fn new(ty: &str, param: &HashMap<String, String>) -> RbhResult<Option<PageQuery>> {
        if ty == "OffsetBased" {
            if let (Some(offset), Some(limit)) = (param.get("offset"), param.get("limit")) {
                let offset = u32::from_str(offset)?;
                let limit = u32::from_str(limit)?;
                return Ok(Some(PageQuery::OffsetBased { offset, limit }));
            }
        } else if ty == "PageBased" {
            if let (Some(number), Some(size)) = (param.get("number"), param.get("size")) {
                let number = u32::from_str(number)?;
                let size = u32::from_str(size)?;
                return Ok(Some(PageQuery::PageBased { number, size }));
            }
        } else if ty == "CursorBased" {
            if let Some(cursor) = param.get("cursor") {
                return Ok(Some(PageQuery::CursorBased(cursor.clone())));
            }
        } else {
            return Err(RabbitholeError::InvalidPageType(ty.to_string()));
        }

        Ok(None)
    }
}

#[derive(Debug)]
pub enum FilterQuery {
    Rsql(HashMap<String, Expr>),
}

impl FilterQuery {
    pub fn new(ty: &str, params: HashMap<String, String>) -> RbhResult<Option<FilterQuery>> {
        if ty == "Rsql" {
            let mut res: HashMap<String, Expr> = Default::default();
            for (k, v) in params.into_iter() {
                let expr = RsqlParser::parse_to_node(&v)?;
                res.insert(k, expr);
            }
            Ok(if res.is_empty() { None } else { Some(FilterQuery::Rsql(res)) })
        } else {
            Err(RabbitholeError::InvalidFilterType(ty.to_string()))
        }
    }
}
