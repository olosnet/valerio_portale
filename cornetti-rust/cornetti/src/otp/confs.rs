use serde::{Deserialize, Deserializer};

fn default_otp_length() -> usize {
    6
}

fn default_otp_expires_minutes() -> i64 {
    10
}

fn default_otp_chars() -> Vec<char> {
    "0123456789".chars().collect()
}

/// Configuration for the simple OTP generator (`[otp]` TOML section).
#[derive(Clone, Debug)]
pub struct SimpleOtpConf {
    /// Length of the generated OTP code (default: 6).
    pub otp_length: usize,
    /// Expiry time in minutes (default: 10).
    pub otp_expires_minutes: i64,
    /// Character set to build the OTP from (default: digits `0-9`).
    pub otp_chars: Vec<char>,
}

impl Default for SimpleOtpConf {
    fn default() -> Self {
        Self {
            otp_length: default_otp_length(),
            otp_expires_minutes: default_otp_expires_minutes(),
            otp_chars: default_otp_chars(),
        }
    }
}

/// Deserializes `otp_chars` from either a single string (`"0123456789"`) or
/// an array of single-character strings (`["0", "1"]`).
fn deserialize_otp_chars<'de, D>(deserializer: D) -> Result<Vec<char>, D::Error>
where
    D: Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum OtpChars {
        Str(String),
        Array(Vec<String>),
    }

    match OtpChars::deserialize(deserializer)? {
        OtpChars::Str(value) => Ok(value.chars().collect()),
        OtpChars::Array(items) => items
            .into_iter()
            .map(|item| {
                let mut chars = item.chars();
                match (chars.next(), chars.next()) {
                    (Some(c), None) => Ok(c),
                    _ => Err(serde::de::Error::custom(format!(
                        "Invalid otp_chars entry '{item}': each array entry must be a single character"
                    ))),
                }
            })
            .collect(),
    }
}

impl<'de> Deserialize<'de> for SimpleOtpConf {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize, Default)]
        #[serde(default)]
        struct Raw {
            otp_length: Option<usize>,
            otp_expires_minutes: Option<i64>,
            #[serde(deserialize_with = "deserialize_otp_chars")]
            otp_chars: Vec<char>,
        }

        let raw = Raw::deserialize(deserializer)?;
        let defaults = SimpleOtpConf::default();

        Ok(SimpleOtpConf {
            otp_length: raw.otp_length.unwrap_or(defaults.otp_length),
            otp_expires_minutes: raw
                .otp_expires_minutes
                .unwrap_or(defaults.otp_expires_minutes),
            otp_chars: if raw.otp_chars.is_empty() {
                defaults.otp_chars
            } else {
                raw.otp_chars
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn otp_conf_from_toml_defaults() {
        let conf: SimpleOtpConf = toml::from_str("").unwrap();
        assert_eq!(conf.otp_length, 6);
        assert_eq!(conf.otp_expires_minutes, 10);
        assert_eq!(conf.otp_chars, vec!['0', '1', '2', '3', '4', '5', '6', '7', '8', '9']);
    }

    #[test]
    fn otp_conf_from_toml_string_chars() {
        let conf: SimpleOtpConf =
            toml::from_str("otp_length = 8\notp_expires_minutes = 5\notp_chars = \"ABCD1234\"")
                .unwrap();
        assert_eq!(conf.otp_length, 8);
        assert_eq!(conf.otp_expires_minutes, 5);
        assert_eq!(conf.otp_chars, vec!['A', 'B', 'C', 'D', '1', '2', '3', '4']);
    }

    #[test]
    fn otp_conf_from_toml_array_chars() {
        let conf: SimpleOtpConf =
            toml::from_str("otp_chars = [\"a\", \"b\", \"c\"]").unwrap();
        assert_eq!(conf.otp_chars, vec!['a', 'b', 'c']);
    }
}
