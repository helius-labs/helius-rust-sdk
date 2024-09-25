use std::{fmt::Display, str::FromStr};

use serde::{Deserialize, Deserializer};

/// Deserialize optional value from a string if it implements [`FromStr`] trait.
pub fn deserialize_opt_from_str<'de, T, D>(deserializer: D) -> Result<Option<T>, D::Error>
where
    T: FromStr,
    T::Err: Display,
    D: Deserializer<'de>,
{
    let value: Option<String> = Option::<String>::deserialize(deserializer)?;

    value.map(|v| v.parse().map_err(serde::de::Error::custom)).transpose()
}
