use serde_json::Value;
use std::str::FromStr;
use std::sync::OnceLock;
use std::{collections::HashMap, env::current_dir, fs};

const JSON_FILENAME: &str = "sample_mcp_messages.json";

static TEST_DATA: OnceLock<TestData> = OnceLock::new();

pub fn get_test_payload(key: &str) -> &String {
    let test_data = TEST_DATA.get_or_init(TestData::new);
    test_data.test_payload(key)
}

pub struct TestData {
    test_payloads: HashMap<String, String>,
}

impl Default for TestData {
    fn default() -> Self {
        Self::new()
    }
}

impl TestData {
    pub fn new() -> Self {
        let data_file = current_dir().unwrap().join("tests/common").join(JSON_FILENAME);

        let data = fs::read_to_string(data_file).expect("Failed to read test data");
        let map: HashMap<String, Value> = json5::from_str(&data).expect("Failed to parse JSON5");

        let mut test_payloads = HashMap::new();
        for (key, value) in map {
            test_payloads.insert(key, serde_json::to_string(&value).expect("Failed to serialize value"));
        }

        Self { test_payloads }
    }

    pub fn test_payload(&self, key: &str) -> &String {
        self.test_payloads.get(key).unwrap()
    }
}

/// Converts a message to a JSON string and then deserializes it back.
/// Tests help to ensures consistent serialization and deserialization across all enum variants.
pub fn re_serialize<T>(message: T) -> T
where
    T: FromStr + serde::Serialize + for<'de> serde::Deserialize<'de>,
    <T as FromStr>::Err: std::fmt::Debug,
{
    //serialize to string
    let message_str = serde_json::to_string(&message).unwrap();

    //deserialize back
    T::from_str(&message_str).unwrap()
}

/// get a test message payload from the sample_mcp_messages.json by key
pub fn get_message<T>(test_payload_key: &str, version: &str) -> T
where
    T: FromStr + for<'de> serde::Deserialize<'de>,
    <T as FromStr>::Err: std::fmt::Debug,
{
    let message_str = get_test_payload(test_payload_key).replace("PROTOCOL_VERSION", version);
    //{"id":13,"jsonrpc":"2.0","method":"tools/list","params":{}}
    T::from_str(&message_str).unwrap()
}
