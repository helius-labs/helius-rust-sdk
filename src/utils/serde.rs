use serde::{Deserialize, Deserializer};
use serde_json::Number;

/// Deserialize optional number as i64
pub fn deserialize_opt_number_to_i64<'de, D>(deserializer: D) -> Result<Option<i64>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Option::<Number>::deserialize(deserializer)?;

    value
        .map(|v| {
            v.as_i64()
                .ok_or_else(|| serde::de::Error::custom("Failed to represent number in i64 range"))
        })
        .transpose()
}
