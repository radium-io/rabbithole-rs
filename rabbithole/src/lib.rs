#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate lazy_static;

use crate::error::RabbitholeError;

pub type RbhResult<T> = Result<T, RabbitholeError>;
pub type RbhOptionRes<T> = Result<Option<T>, RabbitholeError>;

#[macro_use]
pub mod macros;
pub mod entity;
pub mod error;
pub mod model;
pub mod operation;
pub mod rule;
