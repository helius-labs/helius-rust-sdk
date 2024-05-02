use serde::de::Error as SerdeError;
use serde::{Deserialize, Deserializer};
use serde_json::{Number, Value};

/// Deserializes a `String` to a `Number`
pub fn deserialize_str_to_number<'de, D>(deserializer: D) -> Result<Number, D::Error>
where
    D: Deserializer<'de>,
{
    let value: Value = Deserialize::deserialize(deserializer)?;
    match value {
        Value::String(s) => s.parse::<Number>().map_err(SerdeError::custom),
        Value::Number(n) => Ok(n),
        _ => Err(SerdeError::custom("Expected a string or a number")),
    }
}
