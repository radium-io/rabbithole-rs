use crate::model::error;

use crate::entity::SingleEntity;
use crate::query::QuerySettings;
use crate::RbhResult;
use itertools::Itertools;
use std::collections::HashMap;
use std::hash::BuildHasher;
use std::iter::Step;

use std::str::FromStr;

pub trait PageData: Sized + ToString {
    fn page<E: SingleEntity>(
        &self, entities: &[E],
    ) -> RbhResult<(usize, usize, RelativePages<Self>)>;
}

#[derive(Debug)]
pub struct RelativePages<P: PageData> {
    pub first: Option<P>,
    pub last: Option<P>,
    pub prev: Option<P>,
    pub next: Option<P>,
}

impl<P: PageData, S: Default + BuildHasher> From<RelativePages<P>> for HashMap<String, String, S> {
    fn from(relat_pages: RelativePages<P>) -> Self {
        vec![
            ("first", relat_pages.first.as_ref()),
            ("last", relat_pages.last.as_ref()),
            ("prev", relat_pages.prev.as_ref()),
            ("next", relat_pages.next.as_ref()),
        ]
        .into_iter()
        .filter_map(|(k, v)| v.map(|vv| (k.to_string(), vv.to_string())))
        .collect()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
pub struct Cursor {
    pub id: String,
}

impl FromStr for Cursor {
    type Err = error::Error;

    fn from_str(cursor: &str) -> Result<Self, Self::Err> {
        let cursor =
            base64::decode(cursor).map_err(|_| error::Error::InvalidCursorContent(None))?;
        serde_json::from_slice(&cursor).map_err(|_| error::Error::InvalidCursorContent(None))
    }
}

impl ToString for Cursor {
    fn to_string(&self) -> String {
        let cursor = serde_json::to_string(&self).unwrap();
        base64::encode_config(&cursor, base64::URL_SAFE)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
pub struct CursorBasedData {
    pub after: Option<Cursor>,
    pub before: Option<Cursor>,
    pub size: usize,
}

impl ToString for CursorBasedData {
    fn to_string(&self) -> String {
        let after = self
            .after
            .as_ref()
            .map(|c| format!("page[after]={}", c.to_string()))
            .unwrap_or_default();
        let before = self
            .before
            .as_ref()
            .map(|c| format!("page[before]={}", c.to_string()))
            .unwrap_or_default();
        vec![after, before, format!("page[size]={}", self.size)]
            .iter()
            .filter(|s| !s.is_empty())
            .join("&")
    }
}

impl CursorBasedData {
    pub fn new(settings: &QuerySettings, params: &HashMap<String, String>) -> RbhResult<Self> {
        let after = if let Some(after) = params.get("after") {
            Some(after.parse::<Cursor>()?)
        } else {
            None
        };
        let before = if let Some(before) = params.get("before") {
            Some(before.parse::<Cursor>()?)
        } else {
            None
        };
        Ok(Self { after, before, size: settings.default_size })
    }

    fn parse_entity(&self, entity: &impl SingleEntity, is_after: bool) -> Self {
        let cursor = Cursor { id: entity.id() };
        if is_after {
            Self { after: Some(cursor), before: None, size: self.size }
        } else {
            Self { before: Some(cursor), after: None, size: self.size }
        }
    }
}

impl PageData for CursorBasedData {
    fn page<E: SingleEntity>(
        &self, entities: &[E],
    ) -> RbhResult<(usize, usize, RelativePages<Self>)> {
        let after_opt =
            self.after.as_ref().and_then(|cur| entities.iter().position(|r| r.id() == cur.id));
        let before_opt =
            self.before.as_ref().and_then(|cur| entities.iter().position(|r| r.id() == cur.id));
        let first = entities.first().map(|e| self.parse_entity(e, true));
        let last = entities.last().map(|e| self.parse_entity(e, false));

        let (from, to) = match (after_opt, before_opt) {
            (Some(after), Some(before)) if after >= before => {
                return Err(error::Error::BaforeAndAfterCursorNotMatch(None))
            },
            // When the gap between `after` and `before` is larger than `size`
            (Some(after), Some(before)) if before - after > self.size + 1 => {
                (after + 1, after + 1 + self.size)
            },
            (Some(after), Some(before)) => (after + 1, before),
            (Some(after), None) => (after + 1, after + 1 + self.size),
            (None, Some(before)) => (before - self.size, before),
            (None, None) => (0, self.size),
        };
        let prev = entities.get(from).map(|e| self.parse_entity(e, false));
        let next =
            entities.get(to.sub_usize(1).unwrap_or_default()).map(|e| self.parse_entity(e, true));
        Ok((from, to, RelativePages { first, last, prev, next }))
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
pub struct OffsetBasedData {
    pub offset: usize,
    pub limit: usize,
}

impl ToString for OffsetBasedData {
    fn to_string(&self) -> String {
        format!("page[offset]={}&page[limit]={}", self.offset, self.limit)
    }
}

impl OffsetBasedData {
    pub fn new(settings: &QuerySettings, params: &HashMap<String, String>) -> RbhResult<Self> {
        let offset =
            params.get("offset").and_then(|num| usize::from_str(num).ok()).unwrap_or_else(|| 0);
        let limit = params
            .get("limit")
            .and_then(|num| usize::from_str(num).ok())
            .unwrap_or_else(|| settings.default_size);
        Ok(Self { limit, offset })
    }
}

impl PageData for OffsetBasedData {
    fn page<E: SingleEntity>(
        &self, entities: &[E],
    ) -> RbhResult<(usize, usize, RelativePages<Self>)> {
        let start = self.offset.min(entities.len());
        let end = (self.offset + self.limit).min(entities.len());
        let first = Some(OffsetBasedData { offset: 0, limit: self.limit });
        let last = Some(OffsetBasedData {
            offset: entities.len().sub_usize(self.limit).unwrap_or_default(),
            limit: self.limit,
        });
        let prev = Some(OffsetBasedData {
            offset: start.sub_usize(self.limit).unwrap_or_default(),
            limit: self.limit,
        });
        let next = Some(OffsetBasedData { offset: end, limit: self.limit });

        Ok((start, end, RelativePages { first, last, prev, next }))
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
pub struct PageBasedData {
    pub number: usize,
    pub size: usize,
}

impl ToString for PageBasedData {
    fn to_string(&self) -> String {
        format!("page[number]={}&page[size]={}", self.number, self.size)
    }
}

impl PageBasedData {
    pub fn new(settings: &QuerySettings, params: &HashMap<String, String>) -> RbhResult<Self> {
        let number =
            params.get("number").and_then(|num| usize::from_str(num).ok()).unwrap_or_else(|| 0);
        let size = params
            .get("size")
            .and_then(|num| usize::from_str(num).ok())
            .unwrap_or_else(|| settings.default_size);
        Ok(Self { number, size })
    }
}

impl PageData for PageBasedData {
    fn page<E: SingleEntity>(
        &self, entities: &[E],
    ) -> RbhResult<(usize, usize, RelativePages<Self>)> {
        if self.size == 0 {
            return Err(error::Error::InvalidPageSize(None));
        }

        let start = (self.number * self.size).min(entities.len());
        let end = ((self.number + 1) * self.size).min(entities.len());

        let max_page = entities.len() / self.size;

        let first = Some(PageBasedData { number: 0, size: self.size });
        let last = Some(PageBasedData { number: max_page, size: self.size });
        let prev = Some(PageBasedData {
            number: self.number.sub_usize(1).unwrap_or_default(),
            size: self.size,
        });
        let next = Some(PageBasedData { number: (self.number + 1).min(max_page), size: self.size });

        Ok((start, end, RelativePages { first, last, prev, next }))
    }
}

#[derive(Debug)]
pub enum PageQuery {
    OffsetBased(OffsetBasedData),
    PageBased(PageBasedData),
    CursorBased(CursorBasedData),
    None,
}

impl Default for PageQuery {
    fn default() -> Self { Self::None }
}

impl ToString for PageQuery {
    fn to_string(&self) -> String {
        match &self {
            PageQuery::OffsetBased(data) => data.to_string(),
            PageQuery::PageBased(data) => data.to_string(),
            PageQuery::CursorBased(data) => data.to_string(),
            PageQuery::None => Default::default(),
        }
    }
}

impl PageQuery {
    pub fn new(settings: &QuerySettings, params: &HashMap<String, String>) -> RbhResult<PageQuery> {
        match settings.page.ty.as_str() {
            "OffsetBased" => Ok(Self::OffsetBased(OffsetBasedData::new(&settings, params)?)),
            "PageBased" => Ok(Self::PageBased(PageBasedData::new(&settings, params)?)),
            "CursorBased" => Ok(Self::CursorBased(CursorBasedData::new(&settings, params)?)),
            _ => Err(error::Error::InvalidPaginationType(&settings.page.ty, None)),
        }
    }

    pub fn page<'a, E: SingleEntity>(
        &'a self, entities: &'a [E],
    ) -> RbhResult<(&'a [E], HashMap<String, String>)> {
        let (start, end, relat_pages) = match self {
            PageQuery::OffsetBased(data) => {
                let (start, end, relat_pages) = data.page(entities)?;
                (start, end, relat_pages.into())
            },
            PageQuery::PageBased(data) => {
                let (start, end, relat_pages) = data.page(entities)?;
                (start, end, relat_pages.into())
            },
            PageQuery::CursorBased(data) => {
                let (start, end, relat_pages) = data.page(entities)?;
                (start, end, relat_pages.into())
            },
            PageQuery::None => (0, entities.len(), Default::default()),
        };

        Ok((&entities[start.max(0) .. end.min(entities.len())], relat_pages))
    }
}
