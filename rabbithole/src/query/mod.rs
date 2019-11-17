pub mod filter;
pub mod page;
pub mod sort;

use crate::model::error;

use crate::RbhResult;

use crate::query::filter::FilterQuery;
use crate::query::page::PageQuery;
use crate::query::sort::SortQuery;
use percent_encoding::percent_decode_str;
use regex::Regex;
use std::collections::{HashMap, HashSet};

pub type IncludeQuery = HashSet<String>;
pub type FieldsQuery = HashMap<String, HashSet<String>>;

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
    static ref KEY_REGEX: Regex = Regex::new(r#"(?P<name>\w+)\[(?P<param>[\w\-_@]+)\]"#).unwrap();
}

impl Query {
    pub fn from_uri(uri: &http::Uri) -> RbhResult<Query> {
        let mut include_query: IncludeQuery = Default::default();
        let mut include_query_exist = false;
        let mut sort_query: SortQuery = Default::default();
        let mut filter_map: HashMap<String, String> = Default::default();
        let mut filter_type: Option<String> = None;
        let mut fields_map: FieldsQuery = Default::default();
        let mut page_map: HashMap<String, String> = Default::default();

        if let Some(query_str) = uri.query() {
            let query_str = percent_decode_str(query_str)
                .decode_utf8()
                .map_err(|err| error::Error::NotUtf8String(query_str, &err, None))?;

            for (key, value) in query_str.split('&').filter_map(|s| {
                let kv_pair: Vec<&str> = s.splitn(2, '=').collect();
                if kv_pair.len() == 2 && !kv_pair[0].is_empty() {
                    Some((kv_pair[0], kv_pair[1]))
                } else {
                    None
                }
            }) {
                if key == "include" {
                    include_query_exist = true;

                    for v in value.split(',').filter(|s| !s.is_empty()).map(ToString::to_string) {
                        include_query.insert(v);
                    }
                    continue;
                }

                if key == "sort" {
                    sort_query.insert_raw(value)?;
                    continue;
                }

                if let Some(cap) = (&KEY_REGEX as &Regex).captures(key.as_ref()) {
                    if let (Some(name), Some(param)) = (cap.name("name"), cap.name("param")) {
                        let name = name.as_str();
                        let param = param.as_str();

                        if name == "fields" {
                            let values: HashSet<String> = value
                                .split(',')
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
                            if param == "@type" {
                                filter_type = Some(value.into());
                            } else {
                                filter_map.insert(param.into(), value.to_string());
                            }
                        } else if name == "page" {
                            page_map.insert(param.into(), value.to_string());
                        }
                    }
                }
            }
        }

        let include = if include_query_exist { Some(include_query) } else { None };
        let sort = sort_query;
        let page = PageQuery::new(&page_map)?;
        let filter =
            if let Some(ty) = filter_type { FilterQuery::new(&ty, &filter_map)? } else { None };
        let query = Query { include, fields: fields_map, sort, page, filter };
        Ok(query)
    }
}
