use crate::model::error;
use regex::Regex;
use serde::{de, Deserialize, Deserializer, Serialize, Serializer};
use std::str::FromStr;

lazy_static! {
    static ref VERSION_REGEX: Regex = Regex::new(r#"^(?P<major>\d+)\.(?P<minor>\d+)$"#).unwrap();
}

#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct JsonApiVersion {
    pub major: u8,
    pub minor: u8,
}

impl ToString for JsonApiVersion {
    fn to_string(&self) -> String { format!("{}.{}", self.major, self.minor) }
}

impl FromStr for JsonApiVersion {
    type Err = error::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(caps) = VERSION_REGEX.captures(s) {
            if let (Some(major), Some(minor)) = (caps.name("major"), caps.name("minor")) {
                if let (Ok(major), Ok(minor)) =
                    (major.as_str().parse::<u8>(), minor.as_str().parse::<u8>())
                {
                    return Ok(JsonApiVersion { major, minor });
                }
            }
        }
        Err(error::Error::InvalidJsonApiVersion(s.into()))
    }
}

impl Serialize for JsonApiVersion {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> Deserialize<'de> for JsonApiVersion {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        s.parse().map_err(de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use crate::model::version::JsonApiVersion;

    #[test]
    fn deserialize_test() {
        let ver = JsonApiVersion { major: 1, minor: 0 };
        assert_eq!(serde_json::to_string(&ver).unwrap(), "\"1.0\"");
    }

    #[test]
    fn serialize_test() {
        let ver = "1.0";
        let ver = ver.parse::<JsonApiVersion>();
        assert!(ver.is_ok());
        assert_eq!(ver.unwrap(), JsonApiVersion { major: 1, minor: 0 });

        let ver = "1.1";
        let ver = ver.parse::<JsonApiVersion>();
        assert!(ver.is_ok());
        assert_eq!(ver.unwrap(), JsonApiVersion { major: 1, minor: 1 });

        let ver = "1.";
        let ver = ver.parse::<JsonApiVersion>();
        assert!(ver.is_err());

        let ver = "1.a";
        let ver = ver.parse::<JsonApiVersion>();
        assert!(ver.is_err());

        let ver = ".1";
        let ver = ver.parse::<JsonApiVersion>();
        assert!(ver.is_err());

        let ver = ".a";
        let ver = ver.parse::<JsonApiVersion>();
        assert!(ver.is_err());

        let ver = "1.1a";
        let ver = ver.parse::<JsonApiVersion>();
        assert!(ver.is_err());

        let ver = "1.1-alpha1";
        let ver = ver.parse::<JsonApiVersion>();
        assert!(ver.is_err());
    }

    #[test]
    fn ord_test() {
        let jsonapi10 = "1.0".parse::<JsonApiVersion>().unwrap();
        let jsonapi11 = "1.1".parse::<JsonApiVersion>().unwrap();
        assert!(jsonapi10 < jsonapi11);
        assert!(jsonapi10 <= jsonapi11);

        let jsonapi20 = "2.0".parse::<JsonApiVersion>().unwrap();
        let jsonapi19 = "1.9".parse::<JsonApiVersion>().unwrap();
        assert!(jsonapi20 > jsonapi19);
        assert!(jsonapi20 >= jsonapi19);

        let jsonapi11a = "1.1".parse::<JsonApiVersion>().unwrap();
        let jsonapi11b = "1.1".parse::<JsonApiVersion>().unwrap();
        assert_eq!(jsonapi11a, jsonapi11b);
    }
}
