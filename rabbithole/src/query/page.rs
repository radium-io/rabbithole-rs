use crate::model::error;

use crate::RbhResult;
use std::collections::HashMap;

use crate::entity::SingleEntity;

use crate::query::QuerySettings;

use std::str::FromStr;

trait PageData: Sized {
    fn page<E: SingleEntity>(&self, entities: &[E]) -> RbhResult<(usize, usize)>;
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
    pub before: Option<Cursor>,
    pub after: Option<Cursor>,
    pub size: usize,
}

impl CursorBasedData {
    pub fn new(settings: &QuerySettings, params: &HashMap<String, String>) -> RbhResult<Self> {
        let before = if let Some(before) = params.get("before") {
            Some(before.parse::<Cursor>()?)
        } else {
            None
        };
        let after = if let Some(after) = params.get("after") {
            Some(after.parse::<Cursor>()?)
        } else {
            None
        };
        Ok(Self { before, after, size: settings.default_size })
    }
}

impl PageData for CursorBasedData {
    fn page<E: SingleEntity>(&self, entities: &[E]) -> RbhResult<(usize, usize)> {
        let before_opt =
            self.before.as_ref().and_then(|cur| entities.iter().position(|r| r.id() == cur.id));
        let after_opt =
            self.after.as_ref().and_then(|cur| entities.iter().position(|r| r.id() == cur.id));

        match (before_opt, after_opt) {
            (Some(before), Some(after)) if before >= after => {
                Err(error::Error::BaforeAndAfterCursorNotMatch(None))
            },
            // When the gap between `after` and `before` is larger than `size`
            (Some(before), Some(after)) if after - before > self.size + 1 => {
                Ok((before + 1, before + 1 + self.size))
            },
            (Some(before), Some(after)) => Ok((before + 1, after)),
            (Some(before), None) => Ok((before + 1, before + 1 + self.size)),
            (None, Some(after)) => Ok((after - self.size, after)),
            (None, None) => Ok((0, self.size)),
        }
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

impl PageData for OffsetBasedData {
    fn page<E: SingleEntity>(&self, entities: &[E]) -> RbhResult<(usize, usize)> {
        let start = self.offset.min(entities.len());
        let end = (self.offset + self.limit).min(entities.len());
        Ok((start, end))
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

impl PageData for PageBasedData {
    fn page<E: SingleEntity>(&self, entities: &[E]) -> RbhResult<(usize, usize)> {
        let start = (self.number * self.size).min(entities.len());
        let end = ((self.number + 1) * self.size).min(entities.len());
        Ok((start, end))
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

impl PageQuery {
    pub fn new(settings: &QuerySettings, params: &HashMap<String, String>) -> RbhResult<PageQuery> {
        match settings.page.ty.as_str() {
            "OffsetBased" => Ok(Self::OffsetBased(OffsetBasedData::new(&settings, params)?)),
            "PageBased" => Ok(Self::PageBased(PageBasedData::new(&settings, params)?)),
            "CursorBased" => Ok(Self::CursorBased(CursorBasedData::new(&settings, params)?)),
            _ => Err(error::Error::InvalidPaginationType(&settings.page.ty, None)),
        }
    }

    pub fn page<'a, E: SingleEntity>(&'a self, entities: &'a [E]) -> RbhResult<&'a [E]> {
        let (start, end) = match self {
            PageQuery::OffsetBased(data) => data.page(entities)?,
            PageQuery::PageBased(data) => data.page(entities)?,
            PageQuery::CursorBased(data) => data.page(entities)?,
            PageQuery::None => (0, entities.len()),
        };

        Ok(&entities[start.max(0) .. end.min(entities.len())])
    }
}
