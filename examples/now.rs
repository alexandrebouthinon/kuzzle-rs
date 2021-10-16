use kuzzle::{protocols::WebSocket, request, Kuzzle};

#[async_std::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut k = Kuzzle::new(WebSocket::new("localhost", 7512, false));
    k.connect().await?;

    let request = request!({
        "controller": "server",
        "action": "now"
    })?;

    let response = k.query(&request).await?;

    match response.get_result() {
        Some(result) => {
            let now: u64 = serde_json::from_value(result["now"].clone())?;
            println!("Kuzzle current Epoc timestamp: {}", now)
        }
        None => eprintln!("No timestamp was reveived from the Kuzzle server!"),
    }

    k.disconnect().await
}
