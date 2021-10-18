use serde::{Deserialize, Serialize};
use serde_json::value::Value;
use uuid::Uuid;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Request {
    #[serde(rename = "requestId", default = "default_uuid_string")]
    pub request_id: String,
    pub action: String,
    pub controller: String,
    pub index: Option<String>,
    pub collection: Option<String>,
    pub jwt: Option<String>,
    pub body: Option<Value>,
}

fn default_uuid_string() -> String {
    Uuid::new_v4().to_string()
}

#[macro_export]
macro_rules! request {
    ($($json:tt)+) => {
        $crate::forge_request_from_json!($($json)+)
    }
}

#[macro_export(local_inner_macros)]
#[doc(hidden)]
macro_rules! forge_request_from_json {
    ($($json:tt)+) => {{
        let json = serde_json::json!($($json)+);
        serde_json::from_value::<$crate::types::Request>(json)
    }}
}

#[cfg(test)]
mod test {
    use super::*;
    use serde_json::from_value;
    use std::error::Error;
    use uuid::Uuid;

    type BoxResult = Result<(), Box<dyn Error>>;

    #[test]
    fn from_json_macro() -> BoxResult {
        let request: Request = request!({
            "action": "fakeAction",
            "controller": "fakeController",
            "jwt": "fakeToken",
            "body": {
                "query": {
                    "term": 42
                }
            }
        })?;

        assert_eq!("fakeAction", &request.action);
        assert_eq!("fakeController", &request.controller);
        assert_eq!(
            42,
            from_value::<i32>(request.body.unwrap()["query"]["term"].clone())?
        );
        assert!(Uuid::parse_str(&request.request_id).is_ok());
        assert_eq!("fakeToken", &request.jwt.unwrap());

        Ok(())
    }

    #[test]
    fn with_empty_body() -> BoxResult {
        let request: Request = request!({
            "action": "fakeAction",
            "controller": "fakeController",
        })?;

        assert_eq!("fakeAction", &request.action);
        assert_eq!("fakeController", &request.controller);

        Ok(())
    }
}
