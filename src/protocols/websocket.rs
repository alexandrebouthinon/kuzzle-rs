use async_std::io::{Error as IoError, ErrorKind as IoErrorKind};
use async_trait::async_trait;
use async_tungstenite::async_std::connect_async;
use async_tungstenite::async_std::ConnectStream;
use async_tungstenite::tungstenite::error::Error as WsErrors;
use async_tungstenite::tungstenite::protocol::Message;
use async_tungstenite::WebSocketStream;
use futures_util::{sink::SinkExt, stream::StreamExt};
use std::error::Error;
use url::Url;

use super::Protocol;

pub struct WebSocketOptions {
    pub port: u16,
    pub headers: Vec<(String, String)>,
    pub ssl: bool,
    pub reconnection_delay: u64,
    pub ping_interval: u64,
    pub auto_reconnect: bool,
}

impl Default for WebSocketOptions {
    fn default() -> Self {
        Self {
            port: 7512,
            headers: Vec::new(),
            ssl: false,
            reconnection_delay: 1000,
            ping_interval: 2000,
            auto_reconnect: true,
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

    pub fn headers(mut self, headers: Vec<(String, String)>) -> Self {
        self.headers = headers;
        self
    }

    pub fn ssl(mut self, ssl: bool) -> Self {
        self.ssl = ssl;
        self
    }

    pub fn reconnection_delay(mut self, reconnection_delay: u64) -> Self {
        self.reconnection_delay = reconnection_delay;
        self
    }

    pub fn ping_interval(mut self, ping_interval: u64) -> Self {
        self.ping_interval = ping_interval;
        self
    }

    pub fn auto_reconnect(mut self, auto_reconnect: bool) -> Self {
        self.auto_reconnect = auto_reconnect;
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
    ///     .ssl(true)
    ///     .auto_reconnect(true)
    ///     .ping_interval(2000);
    ///
    /// let customized_ws = WebSocket::new("localhost", Some(options));
    /// ```
    pub fn new(host: &str, options: Option<WebSocketOptions>) -> WebSocket {
        WebSocket {
            host: host.into(),
            options: options.unwrap_or(WebSocketOptions::new()),
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
}
