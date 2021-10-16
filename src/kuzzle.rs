use crate::api::{Request, Response};
use crate::protocols::Protocol;

use std::error::Error;

pub struct Kuzzle {
    protocol: Box<dyn Protocol>,
}

impl Kuzzle {
    pub fn new<P: 'static + Protocol>(protocol: P) -> Kuzzle {
        Kuzzle {
            protocol: Box::new(protocol),
        }
    }

    pub async fn connect(&mut self) -> Result<(), Box<dyn Error>> {
        self.protocol.connect().await
    }

    pub async fn disconnect(&mut self) -> Result<(), Box<dyn Error>> {
        self.protocol.disconnect().await
    }

    pub async fn query(&mut self, request: &Request) -> Result<Response, Box<dyn Error>> {
        let response = self.protocol.send(serde_json::to_string(&request)?).await?;
        Ok(serde_json::from_str(&response)?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::request;
    use async_trait::async_trait;

    type BoxResult = Result<(), Box<dyn Error>>;

    #[faux::create]
    pub struct MockedProtocol {}

    #[faux::methods]
    #[allow(unused_parens)]
    #[async_trait]
    impl Protocol for MockedProtocol {
        async fn connect(&mut self) -> Result<(), Box<dyn Error>> {
            todo!()
        }
        async fn disconnect(&mut self) -> Result<(), Box<dyn Error>> {
            todo!()
        }
        async fn send(&mut self, _: String) -> Result<String, Box<dyn Error>> {
            todo!()
        }
    }

    // Quick way to forge fake errors
    fn forge_error() -> Box<dyn Error> {
        Box::new(std::io::Error::last_os_error())
    }

    #[async_std::test]
    async fn should_connect() {
        let mut protocol = MockedProtocol::faux();
        faux::when!(protocol.connect).then(|_| Ok(()));
        let mut kuzzle = Kuzzle::new(protocol);
        assert!(kuzzle.connect().await.is_ok());
    }

    #[async_std::test]
    async fn should_not_connect() {
        let mut protocol = MockedProtocol::faux();
        faux::when!(protocol.connect).then(|_| Err(forge_error()));
        let mut kuzzle = Kuzzle::new(protocol);
        assert!(kuzzle.connect().await.is_err());
    }

    #[async_std::test]
    async fn should_disconnect() {
        let mut protocol = MockedProtocol::faux();
        faux::when!(protocol.disconnect).then(|_| Ok(()));
        let mut kuzzle = Kuzzle::new(protocol);
        assert!(kuzzle.disconnect().await.is_ok());
    }

    #[async_std::test]
    async fn should_not_disconnect() {
        let mut protocol = MockedProtocol::faux();
        faux::when!(protocol.disconnect).then(|_| Err(forge_error()));
        let mut kuzzle = Kuzzle::new(protocol);
        assert!(kuzzle.disconnect().await.is_err());
    }

    #[async_std::test]
    async fn should_query() -> BoxResult {
        let mut protocol = MockedProtocol::faux();
        faux::when!(protocol.send).then(|_| {
            Ok(String::from(
                r#"{"status": 200, "result": { "success": true }}"#,
            ))
        });
        let mut kuzzle = Kuzzle::new(protocol);
        let request: Request = request!({
            "controller": "fakeController",
            "action": "fakeAction"
        })?;

        let response: Response = kuzzle.query(&request).await?;
        let success: bool = serde_json::from_value(response.result.unwrap()["success"].clone())?;
        assert_eq!(response.status, 200);
        assert!(success);
        Ok(())
    }

    #[async_std::test]
    async fn should_not_parse_response() -> BoxResult {
        let mut protocol = MockedProtocol::faux();
        faux::when!(protocol.send).then(|_| Ok(String::from("NOT A VALID JSON STRING")));
        let mut kuzzle = Kuzzle::new(protocol);
        let request = request!({
            "controller": "fakeController",
            "action": "fakeAction"
        })?;

        let result = kuzzle.query(&request).await;
        assert!(result.is_err());
        Ok(())
    }
}
