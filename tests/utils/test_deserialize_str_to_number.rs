use std::fmt;

use helius::utils::deserialize_str_to_number;

use serde::{Deserialize, Serialize};
use serde_json::Number;

#[derive(Debug)]
pub struct TestError(String);

impl fmt::Display for TestError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for TestError {}

impl From<serde_json::Error> for TestError {
    fn from(err: serde_json::Error) -> Self {
        TestError(err.to_string())
    }
}

#[test]
pub fn test_deserialize_str_to_number() -> Result<(), TestError> {
    #[derive(Serialize, Deserialize, Debug)]
    pub struct TestStruct {
        #[serde(deserialize_with = "deserialize_str_to_number")]
        number_str: Number,
    }

    let json_str: &str = r#"{"number_str": "2"}"#;
    let test: TestStruct = serde_json::from_str(json_str)?;
    let number: u64 = test.number_str.as_u64().unwrap();
    assert_eq!(number, 2);

    Ok(())
}
