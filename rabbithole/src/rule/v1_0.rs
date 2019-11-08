use crate::rule::Rule;
use crate::JSON_API_HEADER;

pub(crate) struct ContentTypeMustBeJsonApi;
impl Rule<Option<String>> for ContentTypeMustBeJsonApi {
    fn check(content_type: &Option<String>) -> Result<(), u16> {
        if let Some(content_type) = content_type {
            if content_type == JSON_API_HEADER {
                return Ok(());
            }
        }
        Err(415)
    }
}

pub(crate) struct AcceptHeaderShouldBeJsonApi;
impl Rule<Option<String>> for AcceptHeaderShouldBeJsonApi {
    fn check(accept_header: &Option<String>) -> Result<(), u16> {
        if accept_header.is_some() && accept_header.as_ref().unwrap() == JSON_API_HEADER {
            Ok(())
        } else {
            Err(406)
        }
    }
}
