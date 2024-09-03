use futures_util::StreamExt;
use serde_json::Value;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use tokio_tungstenite::accept_async;

use crate::event::Event;

pub struct Relay {
    events: Arc<Mutex<Vec<Event>>>,
    subscriptions: Arc<Mutex<HashMap<String, Vec<String>>>>,
}

impl Default for Relay {
    fn default() -> Self {
        Relay::new()
    }
}

impl Relay {
    pub fn new() -> Self {
        Relay {
            events: Arc::new(Mutex::new(Vec::new())),
            subscriptions: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn run(&self, addr: &str) -> Result<(), Box<dyn std::error::Error>> {
        let listener = TcpListener::bind(addr).await?;
        println!("Relay listening on: {}", addr);

        while let Ok((stream, _)) = listener.accept().await {
            let ws_stream = accept_async(stream).await.expect("Failed to accept");
            let (mut _write, mut read) = ws_stream.split();
            let events = Arc::clone(&self.events);
            let subscriptions = Arc::clone(&self.subscriptions);

            tokio::spawn(async move {
                while let Some(message) = read.next().await {
                    if let Ok(message) = message {
                        if let Ok(text) = message.into_text() {
                            if let Ok(json) = serde_json::from_str::<Value>(&text) {
                                match json[0].as_str() {
                                    Some("EVENT") => {
                                        if let Ok(event) =
                                            serde_json::from_value::<Event>(json[1].clone())
                                        {
                                            events.lock().await.push(event);
                                            // Broadcast to subscribers
                                        }
                                    }
                                    Some("REQ") => {
                                        if let (Some(subscription_id), Some(_filter)) =
                                            (json[1].as_str(), json[2].as_object())
                                        {
                                            subscriptions
                                                .lock()
                                                .await
                                                .insert(subscription_id.to_string(), vec![]);
                                            // Filter and send matching events
                                        }
                                    }
                                    Some("CLOSE") => {
                                        if let Some(subscription_id) = json[1].as_str() {
                                            subscriptions.lock().await.remove(subscription_id);
                                        }
                                    }
                                    _ => println!("Unknown message type"),
                                }
                            }
                        }
                    }
                }
            });
        }

        Ok(())
    }
}
