<p align="center">
  <img src="https://user-images.githubusercontent.com/7868838/137603738-05d02465-5271-45ea-b5b4-1a026a66d737.png"/>
</p>
<p align="center">
  <a href="https://github.com/alexandrebouthinon/kuzzle-rs/actions/workflows/main.yml">
  <img alt="GitHub Workflow Status" src="https://img.shields.io/github/workflow/status/alexandrebouthinon/kuzzle-rs/Rust?label=workflow&logo=github">
  </a>
  <a href="https://github.com/alexandrebouthinon/kuzzle-rs/blob/master/LICENSE">
    <img src="https://img.shields.io/github/license/alexandrebouthinon/kuzzle-rs.svg?style=flat">
  </a>
</p>

## Usage

The SDK supports different protocols. When instantiating, 
you must choose the protocol to use and fill in the different options needed to connect to Kuzzle.  

```rust
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

    match response.get_result().unwrap() {
        Some(result) => {
            let now: u64 = serde_json::from_value(result["now"].clone())?;
            println!("Kuzzle current Epoc timestamp: {}", now)
        }
        None => eprintln!("No timestamp reveived from the Kuzzle server!"),
    }

    k.disconnect().await
}

```

## About

### Kuzzle

Kuzzle is an open-source backend that includes a scalable server, a multiprotocol API,
an administration console and a set of plugins that provide advanced functionalities like real-time pub/sub, blazing fast search and geofencing.

* :octocat: __[Github](https://github.com/kuzzleio/kuzzle)__
* :earth_africa: __[Website](https://kuzzle.io)__
* :books: __[Documentation](https://docs.kuzzle.io)__
* :email: __[Discord](http://join.discord.kuzzle.io)__
