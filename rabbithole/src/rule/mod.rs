use crate::model::error;
use crate::model::version::JsonApiVersion;

pub mod v1_0;
pub mod v1_1;

pub trait Rule<E> {
    fn check(item: &E) -> Result<(), error::Error>;
}

pub struct RuleDispatcher;

macro_rules! rule_dispatcher {
    ( $($rule_name:ident, $param_type:ty;)* ) => {
            impl RuleDispatcher {
            $(
                #[allow(non_snake_case)]
                pub fn $rule_name(jsonapi_version: &crate::model::version::JsonApiVersion, item: &$param_type) -> Result<(), error::Error> {
                    match jsonapi_version {
                        JsonApiVersion { major: 1, minor: 1 } => v1_1::$rule_name::check(item),
                        _ => v1_0::$rule_name::check(item),
                    }
                }
            )*
            }
        }
}

rule_dispatcher! {
    ContentTypeMustBeJsonApi, Option<String>;
    AcceptHeaderShouldBeJsonApi, Option<String>;
}
