use async_std::io::Error as IoError;
use async_std::io::ErrorKind as IoErrorKind;
use async_trait::async_trait;
use async_tungstenite::async_std::connect_async;
use async_tungstenite::async_std::ConnectStream;
use async_tungstenite::tungstenite::error::Error as WsErrors;
use async_tungstenite::tungstenite::protocol::Message;
use async_tungstenite::WebSocketStream;
use futures_util::sink::SinkExt;
use futures_util::stream::StreamExt;
use std::error::Error;
use url::Url;

use super::Protocol;

pub struct WebSocketOptions {
    pub port: u16,
    pub ssl: bool,
}

impl Default for WebSocketOptions {
    fn default() -> Self {
        Self {
            port: 7512,
            ssl: false,
        }
    }
}

impl WebSocketOptions {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    pub fn ssl(mut self, ssl: bool) -> Self {
        self.ssl = ssl;
        self
    }
}

pub struct WebSocket {
    host: String,
    options: WebSocketOptions,
    stream: Option<WebSocketStream<ConnectStream>>,
}

impl WebSocket {
    /// Create a new WebSocket instance
    ///
    /// # Example
    ///
    /// ```
    /// use kuzzle::protocols::WebSocket;
    ///
    /// // You can rely on the default options...
    /// let websocket = WebSocket::new("localhost", None);
    ///
    /// // ...or make your own configuration
    /// use kuzzle::protocols::WebSocketOptions;
    ///
    /// let options = WebSocketOptions::new()
    ///     .port(7512)
    ///     .ssl(true);
    ///
    /// let customized_ws = WebSocket::new("localhost", Some(options));
    /// ```
    pub fn new(host: &str, options: Option<WebSocketOptions>) -> WebSocket {
        WebSocket {
            host: host.into(),
            options: options.unwrap_or_default(),
            stream: None,
        }
    }

    /// Create and return a valid WebSocket URL using provided host and WebSocketOptions
    ///
    /// # Example
    ///
    /// ```
    /// use kuzzle::protocols::WebSocket;
    ///
    /// let websocket = WebSocket::new("localhost", None);
    /// assert_eq!("ws://localhost:7512", &websocket.get_url());
    ///
    /// use kuzzle::protocols::WebSocketOptions;
    ///
    /// let websocket_ssl = WebSocket::new("localhost", Some(WebSocketOptions::new().ssl(true)));
    /// assert_eq!("wss://localhost:7512", &websocket_ssl.get_url());
    /// ```
    pub fn get_url(&self) -> String {
        match &self.options.ssl {
            true => format!("wss://{}:{}", self.host, self.options.port),
            false => format!("ws://{}:{}", self.host, self.options.port),
        }
    }
}

#[async_trait]
impl Protocol for WebSocket {
    async fn connect(&mut self) -> Result<(), Box<dyn Error>> {
        let url = Url::parse(&self.get_url())?;
        let (ws_stream, _) = connect_async(url).await?;

        self.stream = Some(ws_stream);
        Ok(())
    }

    async fn disconnect(&mut self) -> Result<(), Box<dyn Error>> {
        match self.stream.as_mut() {
            Some(s) => {
                s.close(None).await?;
                self.stream = None;
                Ok(())
            }
            None => Err(Box::new(WsErrors::AlreadyClosed)),
        }
    }

    async fn send(&mut self, request: String) -> Result<String, Box<dyn Error>> {
        match self.stream.as_mut() {
            Some(s) => {
                s.send(Message::Text(request)).await?;
                let res = s.next().await.ok_or_else(|| {
                    Box::new(IoError::new(
                        IoErrorKind::UnexpectedEof,
                        "No response from server",
                    ))
                })??;
                Ok(res.into_text()?)
            }
            None => Err(Box::new(WsErrors::ConnectionClosed)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use surimi::MockServer;

    #[test]
    fn should_forge_ws_url() {
        let ws = WebSocket::new("localhost", None);
        assert_eq!(ws.get_url(), "ws://localhost:7512");
    }

    #[test]
    fn should_forge_wss_url() {
        let ws = WebSocket::new("localhost", Some(WebSocketOptions::new().ssl(true)));
        assert_eq!(ws.get_url(), "wss://localhost:7512");
    }

    #[async_std::test]
    async fn should_not_connect_with_bad_url() {
        let mut ws = WebSocket::new("localhost42", None);
        let result = ws.connect().await;
        assert!(result.is_err());
    }

    #[async_std::test]
    async fn should_disconnect() -> Result<(), Box<dyn Error>> {
        let (_, port) = MockServer::default().start().await?;

        let mut ws = WebSocket::new("localhost", Some(WebSocketOptions::new().port(port)));
        ws.connect().await?;

        assert!(ws.stream.is_some());

        ws.disconnect().await?;
        Ok(())
    }

    #[async_std::test]
    async fn should_not_disconnect_twice() -> Result<(), Box<dyn Error>> {
        let (_, port) = surimi::MockServer::default().start().await?;

        let mut ws = WebSocket::new("localhost", Some(WebSocketOptions::new().port(port)));
        ws.connect().await?;

        assert!(ws.stream.is_some());

        ws.disconnect().await?;
        ws.disconnect().await.err().unwrap();

        Ok(())
    }

    #[async_std::test]
    async fn should_send_request() -> Result<(), Box<dyn Error>> {
        let (_, port) = surimi::MockServer::default()
            .responses(vec![json!({"hello": "world"})])
            .start()
            .await?;

        let mut ws = WebSocket::new("localhost", Some(WebSocketOptions::new().port(port)));
        ws.connect().await?;

        let raw = ws.send("Some request".into()).await?;
        assert_eq!(raw, json!({"hello": "world"}).to_string());

        ws.disconnect().await?;
        Ok(())
    }

    #[async_std::test]
    async fn should_able_to_send_multiple_request() -> Result<(), Box<dyn Error>> {
        let (_, port) = surimi::MockServer::default()
            .responses(vec![
                json!({"hello": "world"}),
                json!({"hello": "world"}),
                json!({"hello": "world"}),
            ])
            .start()
            .await?;

        let mut ws = WebSocket::new("localhost", Some(WebSocketOptions::new().port(port)));
        ws.connect().await?;

        for _ in 0..2 {
            let raw = &ws.send("Trigger some server responses".into()).await?;
            assert_eq!(raw.to_string(), json!({"hello": "world"}).to_string());
        }

        ws.disconnect().await?;
        Ok(())
    }

    #[async_std::test]
    async fn should_not_send_before_connect() -> Result<(), Box<dyn Error>> {
        let (_, port) = surimi::MockServer::default()
            .responses(vec![json!({"hello": "world"})])
            .start()
            .await?;

        let mut ws = WebSocket::new("localhost", Some(WebSocketOptions::new().port(port)));
        let res = ws.send("Some request".into()).await;

        assert!(res.is_err());
        Ok(())
    }

    #[async_std::test]
    async fn should_send_but_no_response() -> Result<(), Box<dyn Error>> {
        let (_, port) = surimi::MockServer::default().start().await?;

        let mut ws = WebSocket::new("localhost", Some(WebSocketOptions::new().port(port)));
        ws.connect().await?;

        let res = ws.send("Some request".into()).await;
        assert!(res.is_err());

        Ok(())
    }
}
