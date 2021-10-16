use async_native_tls::TlsStream;
use async_std::io::{Error as IoError, ErrorKind as IoErrorKind};
use async_std::net::TcpStream;
use async_trait::async_trait;
use async_tungstenite::{
    async_std::connect_async,
    stream::Stream,
    tungstenite::{error::Error as TungsteniteErrors, protocol::Message},
    WebSocketStream,
};
use futures_util::{sink::SinkExt, stream::StreamExt};
use std::error::Error;
use url::Url;

use super::Protocol;

pub struct WebSocket {
    host: String,
    port: u16,
    ssl: bool,
    stream: Option<WebSocketStream<Stream<TcpStream, TlsStream<TcpStream>>>>,
}

impl WebSocket {
    /// Create a new WebSocket instance
    ///
    /// # Example
    ///
    /// ```
    /// use kuzzle::protocols::WebSocket;
    ///
    /// let websocket = WebSocket::new("localhost", 7512, false);
    /// ```
    pub fn new(host: &str, port: u16, ssl: bool) -> WebSocket {
        WebSocket {
            host: host.into(),
            port,
            ssl,
            stream: None,
        }
    }

    /// Create and return a valid WebSocket URL using host, port and SSL configuration
    ///
    /// # Example
    ///
    /// ```
    /// use kuzzle::protocols::WebSocket;
    ///
    /// let websocket = WebSocket::new("localhost", 7512, false);
    /// assert_eq!("ws://localhost:7512", &websocket.get_url());
    /// ```
    pub fn get_url(&self) -> String {
        match &self.ssl {
            true => format!("wss://{}:{}", self.host, self.port),
            false => format!("ws://{}:{}", self.host, self.port),
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
            None => Err(Box::new(TungsteniteErrors::AlreadyClosed)),
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
            None => Err(Box::new(TungsteniteErrors::ConnectionClosed)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn should_forge_ws_url() {
        let ws = WebSocket::new("localhost", 7512, false);
        assert_eq!(ws.get_url(), "ws://localhost:7512");
    }

    #[test]
    fn should_forge_wss_url() {
        let ws = WebSocket::new("localhost", 443, true);
        assert_eq!(ws.get_url(), "wss://localhost:443");
    }

    #[async_std::test]
    async fn should_not_connect_with_bad_url() {
        let mut ws = WebSocket::new("localhost42", 7512, false);
        let result = ws.connect().await;
        assert!(result.is_err());
    }

    #[async_std::test]
    async fn should_not_connect() {
        let mut ws = WebSocket::new("localhost", 4242, false);
        let result = ws.connect().await;
        assert!(result.is_err());
    }
}
