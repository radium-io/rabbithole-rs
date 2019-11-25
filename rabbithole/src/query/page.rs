use crate::model::error;

use crate::RbhResult;
use std::collections::HashMap;

use crate::entity::SingleEntity;
#[cfg(feature = "page_cursor")]
use std::iter::Step;
use std::str::FromStr;

trait PageData: Sized {
    fn new(params: &HashMap<String, String>) -> RbhResult<Option<Self>>;

    fn page<E: SingleEntity>(&self, entities: &[E]) -> (usize, usize);
}

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
pub struct CursorBasedData {
    pub target_id: String,
    pub is_look_after: bool,
    pub limit: usize,
}

impl PageData for CursorBasedData {
    #[cfg(feature = "page_cursor")]
    fn new(params: &HashMap<String, String>) -> RbhResult<Option<Self>> {
        if let Some(cursor) = params.get("cursor") {
            let cursor =
                base64::decode(cursor).map_err(|_| error::Error::InvalidCursorContent(None))?;
            let cursor = String::from_utf8(cursor)
                .map_err(|err| error::Error::InvalidUtf8String(&err, None))?;
            let cursor: CursorBasedData = serde_json::from_str(&cursor)
                .map_err(|_| error::Error::InvalidCursorContent(None))?;

            Ok(Some(cursor))
        } else {
            Ok(None)
        }
    }

    #[cfg(not(feature = "page_cursor"))]
    fn new(params: &HashMap<String, String>) -> RbhResult<Option<Self>> {
        if params.get("cursor").is_some() {
            Err(error::Error::CursorPaginationNotImplemented(None))
        } else {
            Ok(None)
        }
    }

    #[cfg(feature = "page_cursor")]
    fn page<E: SingleEntity>(&self, entities: &[E]) -> (usize, usize) {
        if let Some(tid) = entities.iter().position(|r| r.id() == self.target_id) {
            if self.is_look_after {
                (tid + 1, (tid + 1 + self.limit).min(entities.len()))
            } else {
                ((tid + 1).sub_usize(self.limit).unwrap_or(0usize), tid + 1)
            }
        } else if self.is_look_after {
            (0, self.limit.min(entities.len()))
        } else {
            (entities.len().sub_usize(self.limit).unwrap_or(0usize), entities.len())
        }
    }

    #[cfg(not(feature = "page_cursor"))]
    fn page<E: SingleEntity>(&self, _entities: &[E]) -> (usize, usize) { unimplemented!() }
}

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
pub struct OffsetBasedData {
    pub offset: usize,
    pub limit: usize,
}

impl PageData for OffsetBasedData {
    fn new(params: &HashMap<String, String>) -> RbhResult<Option<Self>> {
        if let (Some(offset), Some(limit)) = (params.get("offset"), params.get("limit")) {
            let offset = usize::from_str(offset).map_err(|_| {
                error::Error::UnmatchedFilterItem("OffsetBased", "offset", offset, None)
            })?;
            let limit = usize::from_str(limit).map_err(|_| {
                error::Error::UnmatchedFilterItem("OffsetBased", "limit", limit, None)
            })?;
            Ok(Some(OffsetBasedData { limit, offset }))
        } else {
            Ok(None)
        }
    }

    fn page<E: SingleEntity>(&self, entities: &[E]) -> (usize, usize) {
        let start = self.offset.min(entities.len());
        let end = (self.offset + self.limit).min(entities.len());
        (start, end)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
pub struct PageBasedData {
    pub number: usize,
    pub size: usize,
}
impl PageData for PageBasedData {
    fn new(params: &HashMap<String, String>) -> RbhResult<Option<Self>> {
        if let (Some(number), Some(size)) = (params.get("number"), params.get("size")) {
            let number = usize::from_str(number).map_err(|_| {
                error::Error::UnmatchedFilterItem("PageBased", "number", number, None)
            })?;
            let size = usize::from_str(size)
                .map_err(|_| error::Error::UnmatchedFilterItem("PageBased", "size", size, None))?;
            Ok(Some(PageBasedData { number, size }))
        } else {
            Ok(None)
        }
    }

    fn page<E: SingleEntity>(&self, entities: &[E]) -> (usize, usize) {
        let start = (self.number * self.size).min(entities.len());
        let end = ((self.number + 1) * self.size).min(entities.len());
        (start, end)
    }
}

#[derive(Debug)]
pub enum PageQuery {
    OffsetBased(OffsetBasedData),
    PageBased(PageBasedData),
    CursorBased(CursorBasedData),
}

impl PageQuery {
    pub fn new(params: &HashMap<String, String>) -> RbhResult<Option<PageQuery>> {
        if let Some(offset) = OffsetBasedData::new(params)? {
            Ok(Some(Self::OffsetBased(offset)))
        } else if let Some(page) = PageBasedData::new(params)? {
            Ok(Some(Self::PageBased(page)))
        } else if let Some(cursor) = CursorBasedData::new(params)? {
            Ok(Some(Self::CursorBased(cursor)))
        } else {
            Ok(None)
        }
    }

    pub fn page<'a, E: SingleEntity>(&'a self, entities: &'a [E]) -> &'a [E] {
        let (start, end) = match self {
            PageQuery::OffsetBased(data) => data.page(entities),
            PageQuery::PageBased(data) => data.page(entities),
            PageQuery::CursorBased(data) => data.page(entities),
        };

        &entities[start .. end]
    }
}

#[cfg(test)]
mod tests {
    use crate::query::page::{CursorBasedData, PageQuery};
    use crate::query::QuerySettings;
    use percent_encoding::{percent_encode, NON_ALPHANUMERIC};

    #[test]
    fn cursor_des_test() {
        let query = QuerySettings { filter_type: "Rsql".to_string() };

        let ori_cursor =
            CursorBasedData { target_id: "target_id".to_string(), is_look_after: true, limit: 10 };

        let ori_cursor_str: String = serde_json::to_string(&ori_cursor).unwrap();
        let ori_cursor_str = base64::encode_config(&ori_cursor_str, base64::URL_SAFE_NO_PAD);
        let uri = format!("page[cursor]={}", ori_cursor_str);
        let uri = percent_encode(uri.as_bytes(), NON_ALPHANUMERIC);
        let uri = format!("/?{}", uri.to_string());
        let uri: http::Uri = uri.parse().unwrap();
        let query = query.from_uri(&uri).unwrap();
        if let Some(PageQuery::CursorBased(cursor)) = query.page {
            assert_eq!(cursor, ori_cursor);
        } else {
            unreachable!();
        }
    }
}
