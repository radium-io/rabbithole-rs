use rabbithole::model::version::JsonApiVersion;
use rabbithole::query::QuerySettings;
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct ActixSettingsModel {
    pub host: String,
    pub port: u32,
    pub path: String,
    pub jsonapi: JsonApiSettings,
    pub query: QuerySettings,
}

#[derive(Debug, Deserialize, Clone)]
pub struct JsonApiSettings {
    pub version: JsonApiVersion,
}
