use crate::model::error;
use crate::rule::Rule;
use crate::JSON_API_HEADER;

pub(crate) struct ContentTypeMustBeJsonApi;
impl Rule<Option<String>> for ContentTypeMustBeJsonApi {
    fn check(content_type: &Option<String>) -> Result<(), error::Error> {
        if let Some(content_type) = content_type {
            if content_type == JSON_API_HEADER {
                return Ok(());
            }
        }
        Err(error::Error::InvalidContentType(
            &format!("`{}`", JSON_API_HEADER),
            content_type.as_deref().unwrap_or("nothing"),
            None,
        ))
    }
}

pub(crate) struct AcceptHeaderShouldBeJsonApi;
impl Rule<Option<String>> for AcceptHeaderShouldBeJsonApi {
    fn check(accept_header: &Option<String>) -> Result<(), error::Error> {
        if accept_header.is_some() && accept_header.as_ref().unwrap() == JSON_API_HEADER {
            Ok(())
        } else {
            Err(error::Error::InvalidAccept(
                &format!("`{}`", JSON_API_HEADER),
                accept_header.as_deref().unwrap_or("nothing"),
                None,
            ))
        }
    }
}
