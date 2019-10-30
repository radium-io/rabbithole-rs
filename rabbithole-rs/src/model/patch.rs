use crate::model::relationship::Relationships;
use crate::model::resource::ResourceAttributes;
use crate::model::Id;

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct Patch {
    pub resource_type: String,
    pub resource_id: Id,
    pub item: PatchData,
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PatchData {
    Relationships(Relationships),
    Attributes(ResourceAttributes),
}
