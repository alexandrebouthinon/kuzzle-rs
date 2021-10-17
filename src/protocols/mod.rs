use async_trait::async_trait;
use std::error::Error as Errors;

pub mod websocket;
pub use self::websocket::{WebSocket, WebSocketOptions};

#[async_trait]
pub trait Protocol {
    async fn connect(&mut self) -> Result<(), Box<dyn Errors>>;
    async fn disconnect(&mut self) -> Result<(), Box<dyn Errors>>;
    async fn send(&mut self, request: String) -> Result<String, Box<dyn Errors>>;
}
