#![feature(impl_trait_in_bindings)]

#[macro_use]
extern crate serde_derive;
#[macro_use]
extern crate lazy_static;

use crate::error::RabbitholeError;

pub type RbhResult<T> = std::result::Result<T, RabbitholeError>;

pub mod entity;
pub mod error;
pub mod model;
