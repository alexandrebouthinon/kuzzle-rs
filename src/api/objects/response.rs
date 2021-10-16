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

impl Response {
    pub fn get_error(&self) -> Option<&Value> {
        self.error.as_ref()
    }

    pub fn get_result(&self) -> Option<&Value> {
        self.result.as_ref()
    }

    pub fn get_status(&self) -> &u16 {
        &self.status
    }

    pub fn get_volatile(&self) -> Option<&Value> {
        self.volatile.as_ref()
    }
}
