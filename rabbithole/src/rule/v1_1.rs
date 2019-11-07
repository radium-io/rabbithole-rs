use crate::rule::Rule;
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
    fn check(content_type: &Option<String>) -> Result<(), u16> { check_header(content_type, 415) }
}

pub(crate) struct AcceptHeaderShouldBeJsonApi;
impl Rule<Option<String>> for AcceptHeaderShouldBeJsonApi {
    fn check(accept_header: &Option<String>) -> Result<(), u16> { check_header(accept_header, 406) }
}

fn check_header(item: &Option<String>, error_code: u16) -> Result<(), u16> {
    if let Some(item) = item {
        let params = extract_params_of_media_type(item);
        if item.starts_with("application/vnd.api+json")
            && (has_no_param(&params) || has_only_profile_param(&params))
        {
            return Ok(());
        }
    }
    Err(error_code)
}
