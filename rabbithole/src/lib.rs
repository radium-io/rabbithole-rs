#![feature(associated_type_defaults)]
#![feature(step_trait)]

#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate lazy_static;

use crate::model::error::Error;

pub type RbhResult<T> = Result<T, Error>;
pub type RbhOptionRes<T> = Result<Option<T>, Error>;
pub const JSON_API_HEADER: &str = "application/vnd.api+json";

pub mod entity;
pub mod model;
pub mod operation;
pub mod query;
pub mod rule;
