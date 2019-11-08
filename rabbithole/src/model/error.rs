use crate::model::link::{Link, Links, RawUri};
use crate::model::{Id, Meta};
use serde::{Deserialize, Serialize};
use std::fmt;

pub type Errors = Vec<Error>;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq, Default)]
pub struct ErrorLinks {
    #[serde(skip_serializing_if = "Option::is_none")]
    about: Option<Link>,
    #[serde(flatten)]
    #[serde(skip_serializing_if = "Links::is_empty")]
    links: Links,
}

impl ErrorLinks {
    pub fn is_empty(&self) -> bool { self.about.is_none() && self.links.is_empty() }
}

impl From<Links> for ErrorLinks {
    fn from(mut links: Links) -> Self {
        let about = links.remove("about");
        Self { about, links }
    }
}

/// Error location
#[derive(Debug, Clone, Default, Eq, PartialEq, Serialize, Deserialize)]
pub struct ErrorSource {
    pub pointer: Option<RawUri>,
    pub parameter: Option<String>,
}

/// JSON-API Error
/// All fields are optional
#[derive(Serialize, Deserialize, Debug, Clone, Default, PartialEq)]
pub struct Error {
    /// a unique identifier for this particular occurrence of the problem
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<Id>,
    #[serde(skip_serializing_if = "ErrorLinks::is_empty")]
    pub links: ErrorLinks,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    /// an application-specific error code, expressed as a string value
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    /// a short, human-readable summary of the problem that
    /// SHOULD NOT change from occurrence to occurrence of the problem,
    /// except for purposes of localization
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<ErrorSource>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub meta: Option<Meta>,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "title: {}, detail: {}",
            self.title.as_deref().unwrap_or(""),
            self.detail.as_deref().unwrap_or("")
        )
    }
}

/// Rabbithole Error Code:
///   1. Magic Word(0..4): Fixed "RBH-", to indicate User that this error is from Rabbithole Server,
///                        rather than an application-specific error
///   2. Domain Code(4..5): Two digits to indicate where the error is:
///     0. "00": Miscellaneous part (general errors)
///     1. "01": URL Query part
///     2. "02": Fields of HTTP Body
///     3. "03": HTTP Header part
///   3. Specific Code(5..6): Two digits to indicate the more info about the location, just as the `title` said
macro_rules! rabbithole_errors_inner {
    ( $(ty: $ty:ident, status: $status:expr, code: $code:expr, title: $title:expr, detail: $detail:expr, param: [$($param_arg:ident: $param_ty:ty),*];)* ) => {
        $(
            #[allow(non_snake_case)]
            pub fn $ty($($param_arg: $param_ty),*) -> Error {
                Self {
                    id: Some(uuid::Uuid::new_v4().to_string()),
                    status: Some($status.as_str().into()),
                    code: Some($code.into()),
                    title: Some($title.into()),
                    detail: Some(format!($detail, $($param_arg = $param_arg),*)),
                    ..Default::default()
                }
            }
        )*
    };

    ( $(ty: $ty:ident, code: $code:expr, title: $title:expr, detail: $detail:expr, param: [$($param_arg:ident: $param_ty:ty),*];)* ) => {
        rabbithole_errors!($(ty: $ty, status: http::StatusCode::NOT_ACCEPTABLE, code: $code, title: $title, detail: $detail, param: [$($param_arg: $param_ty),*];)*);
    };
}

macro_rules! rabbithole_errors {
    ( $(ty: $ty:ident, code: $code:expr, title: $title:expr, detail: $detail:expr, param: [$($param_arg:ident: $param_ty:ty),*];)* ) => {
        impl Error {
            rabbithole_errors_inner!($(ty: $ty, status: http::StatusCode::NOT_ACCEPTABLE, code: $code, title: $title, detail: $detail, param: [$($param_arg: $param_ty),*];)*);
        }
    };

    ( $(ty: $ty:ident, status: $status:expr, code: $code:expr, title: $title:expr, detail: $detail:expr, param: [$($param_arg:ident: $param_ty:ty),*];)* ) => {
        impl Error {
            rabbithole_errors_inner!($(ty: $ty, status: $status, code: $code, title: $title, detail: $detail, param: [$($param_arg: $param_ty),*];)*);
        }
    };
}

rabbithole_errors! {
    ty: InvalidUtf8String,
    code: "RBH-0001",
    title: "Invalid UTF-8 String",
    detail: "The String {invalid} is not a valid UTF-8 String",
    param: [invalid: &str];

    ty: InvalidPageType,
    code: "RBH-0101",
    title: "Invalid Page Type",
    detail: r#"Invalid page type: {invalid}, the valid ones are: ["OffsetBased", "PageBased", "CursorBased"]"#,
    param: [invalid: &str];

    ty: InvalidFilterType,
    code: "RBH-0102",
    title: "Invalid Filter Type",
    detail: r#"Invalid filter type: {invalid}, the valid ones are: ["Rsql"]"#,
    param: [invalid: &str];

    ty: UnmatchedFilterItem,
    code: "RBH-0103",
    title: "Unmatched Filter Item",
    detail: "Filter type [{filter_type}] and filter item [{filter_key} = {filter_value}] are not matched",
    param: [filter_type: &str, filter_key: &str, filter_value: &str];

    ty: InvalidJsonApiVersion,
    code: "RBH-0201",
    title: "Invalid JSON API Version",
    detail: "A invalid JSON:API version: {invalid_version}",
    param: [invalid_version: String];

    ty: InvalidContentType,
    code: "RBH-0301",
    title: "Invalid Content-Type Header",
    detail: "The `Content-Type` header of Request must be {header_hint}, but {invalid_header} found",
    param: [header_hint: &str, invalid_header: &str];

    ty: InvalidAccept,
    code: "RBH-0301",
    title: "Invalid Accept Header",
    detail: "The `Accept` header of Request must be {header_hint}, but {invalid_header} found",
    param: [header_hint: &str, invalid_header: &str];
}
