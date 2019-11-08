use crate::model::error;
use crate::rule::Rule;
use crate::JSON_API_HEADER;
use std::collections::HashMap;

fn extract_params_of_media_type(media_type: &str) -> HashMap<String, String> {
    let mut params: HashMap<String, String> = Default::default();
    for param in media_type.split(';').skip(1) {
        let param: Vec<&str> = param.split('=').map(|s| s.trim()).collect();
        if param.len() == 2 {
            params.insert(param[0].into(), param[1].into());
        }
    }

    params
}

fn has_no_param(params: &HashMap<String, String>) -> bool { params.is_empty() }

fn has_only_profile_param(params: &HashMap<String, String>) -> bool {
    params.len() == 1 && params.contains_key("profile")
}

pub(crate) struct ContentTypeMustBeJsonApi;
impl Rule<Option<String>> for ContentTypeMustBeJsonApi {
    fn check(content_type: &Option<String>) -> Result<(), error::Error> {
        if is_valid(&content_type) {
            Ok(())
        } else {
            Err(error::Error::InvalidContentType(
                &format!("`{}` with optional `profile` parameter", JSON_API_HEADER),
                content_type.as_deref().unwrap_or("nothing"),
            ))
        }
    }
}

pub(crate) struct AcceptHeaderShouldBeJsonApi;
impl Rule<Option<String>> for AcceptHeaderShouldBeJsonApi {
    fn check(accept_header: &Option<String>) -> Result<(), error::Error> {
        if is_valid(&accept_header) {
            Ok(())
        } else {
            Err(error::Error::InvalidAccept(
                &format!("`{}` with optional `profile` parameter", JSON_API_HEADER),
                accept_header.as_deref().unwrap_or("nothing"),
            ))
        }
    }
}

fn is_valid(item: &Option<String>) -> bool {
    if let Some(item) = item {
        let params = extract_params_of_media_type(item);
        if item.starts_with(JSON_API_HEADER)
            && (has_no_param(&params) || has_only_profile_param(&params))
        {
            return true;
        }
    }
    false
}
