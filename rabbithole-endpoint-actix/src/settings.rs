use rabbithole::model::version::JsonApiVersion;
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct ActixSettingsModel {
    pub domain: String,
    pub suffix: String,
    pub jsonapi: JsonApiSettings,
}

#[derive(Debug, Deserialize, Clone)]
pub struct JsonApiSettings {
    pub version: JsonApiVersion,
}
