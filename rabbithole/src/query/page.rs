use crate::model::error;

use crate::entity::SingleEntity;
use crate::query::{encode_string, QuerySettings};
use crate::RbhResult;
use itertools::Itertools;
use num_integer::Integer;
use std::collections::HashMap;
use std::hash::BuildHasher;
use std::iter::Step;
use std::str::FromStr;

pub trait PageData: Sized + Into<PageQuery> + Clone {
    fn page<E: SingleEntity>(
        &self, entities: &[E],
    ) -> RbhResult<(usize, usize, RelativePages<Self>)>;

    fn to_string(&self, raw_encode: bool) -> String;
}

#[derive(Debug)]
pub struct RelativePages<P: PageData> {
    pub first: Option<P>,
    pub last: Option<P>,
    pub prev: Option<P>,
    pub next: Option<P>,
}

impl<P: PageData> Default for RelativePages<P> {
    fn default() -> Self { Self { first: None, last: None, prev: None, next: None } }
}

impl<P: PageData, S: Default + BuildHasher> From<RelativePages<P>>
    for HashMap<String, PageQuery, S>
{
    fn from(relat_pages: RelativePages<P>) -> Self {
        vec![
            ("first", relat_pages.first.as_ref()),
            ("last", relat_pages.last.as_ref()),
            ("prev", relat_pages.prev.as_ref()),
            ("next", relat_pages.next.as_ref()),
        ]
        .into_iter()
        .filter_map(|(k, v)| v.map(|vv| (k.to_string(), vv.clone().into())))
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
        let size = params
            .get("size")
            .and_then(|s| usize::from_str(s.as_str()).ok())
            .unwrap_or(settings.default_size);
        Ok(Self { after, before, size })
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

impl From<CursorBasedData> for PageQuery {
    fn from(data: CursorBasedData) -> Self { PageQuery::CursorBased(data) }
}

impl PageData for CursorBasedData {
    fn page<E: SingleEntity>(
        &self, entities: &[E],
    ) -> RbhResult<(usize, usize, RelativePages<Self>)> {
        if self.size == 0 {
            return Err(error::Error::InvalidPageSize(None));
        }

        let after_opt =
            self.after.as_ref().and_then(|cur| entities.iter().position(|r| r.id() == cur.id));
        let before_opt =
            self.before.as_ref().and_then(|cur| entities.iter().position(|r| r.id() == cur.id));

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
            (None, Some(before)) => (before.sub_usize(self.size).unwrap_or_default(), before),
            (None, None) => (0, self.size),
        };
        let (from, to) = (from.max(0), to.min(entities.len()));

        let prev =
            if from != 0 { entities.get(from).map(|e| self.parse_entity(e, false)) } else { None };
        let next = if to != entities.len() {
            entities.get(to.sub_usize(1).unwrap_or_default()).map(|e| self.parse_entity(e, true))
        } else {
            None
        };
        Ok((from, to, RelativePages { first: None, last: None, prev, next }))
    }

    fn to_string(&self, raw_encode: bool) -> String {
        let after = self
            .after
            .as_ref()
            .map(|c| format!("{}={}", encode_string("page[after]", raw_encode), c.to_string()))
            .unwrap_or_default();
        let before = self
            .before
            .as_ref()
            .map(|c| format!("{}={}", encode_string("page[before]", raw_encode), c.to_string()))
            .unwrap_or_default();
        vec![after, before, format!("{}={}", encode_string("page[size]", raw_encode), self.size)]
            .iter()
            .filter(|s| !s.is_empty())
            .join("&")
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
pub struct OffsetBasedData {
    pub offset: usize,
    pub limit: usize,
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

impl From<OffsetBasedData> for PageQuery {
    fn from(data: OffsetBasedData) -> Self { PageQuery::OffsetBased(data) }
}

impl PageData for OffsetBasedData {
    fn page<E: SingleEntity>(
        &self, entities: &[E],
    ) -> RbhResult<(usize, usize, RelativePages<Self>)> {
        if entities.is_empty() {
            return Ok((0, 0, RelativePages::default()));
        }

        let start = self.offset.min(entities.len()).max(0);
        let end = (self.offset + self.limit).min(entities.len());

        eprintln!("offset: {}, limit: {}, start: {}, end: {}", self.offset, self.limit, start, end);

        let first = Some(OffsetBasedData { offset: 0, limit: self.limit });
        let last = match entities.len().sub_usize(self.limit) {
            Some(offset) if offset > 0 => Some(OffsetBasedData { offset, limit: self.limit }),
            _ => None,
        };

        let prev = match start.sub_usize(self.limit) {
            Some(offset) if offset > 0 => Some(OffsetBasedData { offset, limit: self.limit }),
            _ => None,
        };

        let next = if end == entities.len() {
            None
        } else {
            Some(OffsetBasedData { offset: end, limit: self.limit })
        };

        Ok((start, end, RelativePages { first, last, prev, next }))
    }

    fn to_string(&self, raw_encode: bool) -> String {
        format!(
            "{}={}&{}={}",
            encode_string("page[offset]", raw_encode),
            self.offset,
            encode_string("page[limit]", raw_encode),
            self.limit
        )
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Eq, PartialEq)]
pub struct PageBasedData {
    pub number: usize,
    pub size: usize,
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

impl From<PageBasedData> for PageQuery {
    fn from(data: PageBasedData) -> Self { PageQuery::PageBased(data) }
}

impl PageData for PageBasedData {
    fn page<E: SingleEntity>(
        &self, entities: &[E],
    ) -> RbhResult<(usize, usize, RelativePages<Self>)> {
        if entities.is_empty() {
            return Ok((0, 0, RelativePages::default()));
        }

        if self.size == 0 {
            return Err(error::Error::InvalidPageSize(None));
        }

        let start = (self.number * self.size).min(entities.len());
        let end = ((self.number + 1) * self.size).min(entities.len());

        let max_page = entities.len().div_ceil(&self.size).sub_usize(1).unwrap_or_default();

        let first = Some(PageBasedData { number: 0, size: self.size });
        let last = if max_page == 0 {
            None
        } else {
            Some(PageBasedData { number: max_page, size: self.size })
        };
        let prev = self.number.sub_usize(1).map(|number| PageBasedData { number, size: self.size });
        let next = if self.number + 1 > max_page {
            None
        } else {
            Some(PageBasedData { number: self.number + 1, size: self.size })
        };

        Ok((start, end, RelativePages { first, last, prev, next }))
    }

    fn to_string(&self, raw_encode: bool) -> String {
        format!(
            "{}={}&{}={}",
            encode_string("page[number]", raw_encode),
            self.number,
            encode_string("page[size]", raw_encode),
            self.size
        )
    }
}

#[derive(Debug, Clone)]
pub enum PageQuery {
    OffsetBased(OffsetBasedData),
    PageBased(PageBasedData),
    CursorBased(CursorBasedData),
}

impl PageQuery {
    pub fn new(settings: &QuerySettings, params: &HashMap<String, String>) -> RbhResult<PageQuery> {
        if let Some(page_settings) = settings.page.as_ref() {
            match page_settings.ty.as_str() {
                "OffsetBased" => Ok(Self::OffsetBased(OffsetBasedData::new(&settings, params)?)),
                "PageBased" => Ok(Self::PageBased(PageBasedData::new(&settings, params)?)),
                "CursorBased" => Ok(Self::CursorBased(CursorBasedData::new(&settings, params)?)),
                _ => Err(error::Error::InvalidPaginationType(&page_settings.ty, None)),
            }
        } else {
            Err(error::Error::InvalidPaginationType(&"None", None))
        }
    }

    pub fn page<'a, E: SingleEntity>(
        &'a self, entities: &'a [E],
    ) -> RbhResult<(&'a [E], HashMap<String, PageQuery>)> {
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
        };

        Ok((&entities[start .. end], relat_pages))
    }

    pub fn to_string(&self, raw_encode: bool) -> String {
        match &self {
            PageQuery::OffsetBased(data) => data.to_string(raw_encode),
            PageQuery::PageBased(data) => data.to_string(raw_encode),
            PageQuery::CursorBased(data) => data.to_string(raw_encode),
        }
    }
}
