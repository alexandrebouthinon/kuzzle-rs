use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Response {
    #[serde(rename = "requestId")]
    pub request_id: String,
    pub status: u16,
    pub node: Option<String>,
    pub action: String,
    pub controller: String,
    pub index: Option<String>,
    pub collection: Option<String>,
    pub error: Option<Value>,
    pub result: Option<Value>,
    pub volatile: Option<Value>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_response_serde() {
        let response_str = String::from(
            r#"{
                "requestId":"0",
                "status":200,
                "node":"foo",
                "action":"bar",
                "controller":"baz",
                "index":"qux",
                "collection":"quux",
                "error": {
                    "message":"error message",
                    "code":1
                }
            }"#,
        );

        let response: Response = serde_json::from_str(&response_str).unwrap();

        assert_eq!(response.request_id, "0");
        assert_eq!(response.status, 200);
        assert_eq!(response.node, Some(String::from("foo")));
        assert_eq!(response.action, String::from("bar"));
        assert_eq!(response.controller, String::from("baz"));
        assert_eq!(response.index, Some(String::from("qux")));
        assert_eq!(response.collection, Some(String::from("quux")));
        assert_eq!(
            response.error,
            Some(json!({"message":"error message", "code":1 }))
        );
        assert_eq!(response.result, None);
        assert_eq!(response.volatile, None);
    }
}
