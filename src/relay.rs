use futures_util::{SinkExt, StreamExt};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use url::Url;

pub struct Relay {
    url: Url,
}

impl Relay {
    pub fn new(url: &str) -> Result<Self, url::ParseError> {
        Ok(Self {
            url: Url::parse(url)?,
        })
    }

    pub async fn connect(&self) -> Result<(), Box<dyn std::error::Error>> {
        let (ws_stream, _) = connect_async(&self.url.to_string()).await?;
        let (mut write, mut read) = ws_stream.split();

        // Example: Send a subscription request
        let subscription = r#"["REQ", "my_subscription_id", {"kinds": [1], "limit": 10}]"#;
        write.send(Message::Text(subscription.to_string())).await?;

        // Handle incoming messages
        while let Some(message) = read.next().await {
            match message {
                Ok(Message::Text(text)) => println!("{}", text),
                Ok(Message::Binary(data)) => println!("{:?}", data),
                Err(e) => eprintln!("Error: {}", e),
                _ => {}
            }
        }

        Ok(())
    }
}
