pub mod dog;
pub mod human;
use rabbithole::model::error;

lazy_static! {
    static ref WRONG_FIELD_TYPE: error::Error = error::Error {
        status: Some("400".into()),
        code: Some("1".into()),
        title: Some("Wrong field type".into()),
        ..Default::default()
    };
    static ref ENTITY_NOT_FOUND: error::Error = error::Error {
        status: Some("400".into()),
        code: Some("2".into()),
        title: Some("Entity not found".into()),
        ..Default::default()
    };
    static ref INVALID_IDS_CONTAINED: error::Error = error::Error {
        status: Some("400".into()),
        code: Some("3".into()),
        title: Some("Invalid IDs Contained".into()),
        ..Default::default()
    };
}
