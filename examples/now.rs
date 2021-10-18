use kuzzle::protocols::WebSocket;
use kuzzle::{request, Kuzzle};

#[async_std::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut k = Kuzzle::new(WebSocket::new("localhost", None));
    k.connect().await?;

    let request = request!({
        "controller": "server",
        "action": "now"
    })?;

    let response = k.query(&request).await?;

    match response.result {
        Some(result) => println!("Kuzzle current Epoc timestamp: {}", &result["now"]),
        None => eprintln!("No timestamp was reveived from the Kuzzle server!"),
    }

    k.disconnect().await
}
