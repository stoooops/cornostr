use crate::crypto::{generate_keypair, sign_event, verify_event};
use crate::event::Event;
use futures_util::{SinkExt, StreamExt};
use secp256k1::Keypair;
use std::collections::HashMap;
use tokio_tungstenite::connect_async;
use url::Url;

/// Represents a Nostr client that can connect to relays, publish events, and manage subscriptions.
pub struct Client {
    /// The client's keypair for signing events. It's optional because a client might not always have a keypair set.
    keypair: Option<Keypair>,
    /// A map of relay URLs to their corresponding WebSocket connections.
    relays: HashMap<
        String,
        tokio_tungstenite::WebSocketStream<
            tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
        >,
    >,
    /// A map of subscription IDs to the events received for that subscription.
    subscriptions: HashMap<String, Vec<Event>>,
}

impl Default for Client {
    fn default() -> Self {
        Client::new()
    }
}

impl Client {
    /// Creates a new Client instance with no keypair, relays, or subscriptions.
    pub fn new() -> Self {
        Client {
            keypair: None,
            relays: HashMap::new(),
            subscriptions: HashMap::new(),
        }
    }

    /// Sets the client's keypair for signing events.
    #[allow(dead_code)]
    pub fn set_keypair(&mut self, keypair: Keypair) {
        self.keypair = Some(keypair);
    }

    /// Generates a new keypair for the client using the crypto module's generate_keypair function.
    pub fn generate_keypair(&mut self) {
        self.keypair = Some(generate_keypair());
    }

    /// Connects to a Nostr relay at the given URL.
    ///
    /// This method establishes a WebSocket connection to the relay and stores it in the relays map.
    pub async fn connect(&mut self, relay_url: &str) -> Result<(), Box<dyn std::error::Error>> {
        let url = Url::parse(relay_url)?;
        let (ws_stream, _) = connect_async(url.to_string()).await?;
        self.relays.insert(relay_url.to_string(), ws_stream);
        Ok(())
    }

    /// Publishes an event to all connected relays.
    ///
    /// This method signs the event with the client's keypair (if set), then sends it to all connected relays.
    #[allow(dead_code)]
    pub async fn publish_event(
        &mut self,
        event: &mut Event,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(keypair) = &self.keypair {
            // Sign the event
            event.sig = sign_event(event, keypair);

            // Create a JSON array with "EVENT" and the event
            let message = serde_json::json!(["EVENT", event]);

            // Convert the entire structure to a string
            let message_string = serde_json::to_string(&message)?;

            // Send the message to all connected relays
            for ws_stream in self.relays.values_mut() {
                ws_stream
                    .send(tokio_tungstenite::tungstenite::Message::Text(
                        message_string.clone(),
                    ))
                    .await?;
            }
            Ok(())
        } else {
            Err("No keypair set".into())
        }
    }

    /// Creates a new subscription with the given ID and filter.
    ///
    /// This method sends a subscription request to all connected relays and initializes
    /// an empty vector in the subscriptions map to store future events for this subscription.
    pub async fn subscribe(
        &mut self,
        subscription_id: &str,
        filter: &str,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // Prepare the subscription message in the format expected by relays: ["REQ", <subscription_id>, <filter>]
        let message = format!("[\"{}\", \"{}\", {}]", "REQ", subscription_id, filter);
        // Send the subscription request to all connected relays
        for ws_stream in self.relays.values_mut() {
            ws_stream
                .send(tokio_tungstenite::tungstenite::Message::Text(
                    message.clone(),
                ))
                .await?;
        }
        // Initialize an empty vector for this subscription to store future events
        self.subscriptions
            .insert(subscription_id.to_string(), Vec::new());
        Ok(())
    }

    /// Receives and processes events from all connected relays.
    ///
    /// This method listens for incoming messages from all relays, verifies received events,
    /// and stores them in the appropriate subscription's event list.
    pub async fn receive_events(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        for ws_stream in self.relays.values_mut() {
            while let Some(message) = ws_stream.next().await {
                let message = message?;
                match message {
                    tokio_tungstenite::tungstenite::Message::Text(text) => {
                        // Parse the incoming message as JSON
                        let json: serde_json::Value = serde_json::from_str(&text)?;
                        // Print the text as pretty JSON
                        println!("{}", serde_json::to_string_pretty(&json)?);
                        // Check if the message is an event message
                        if json[0] == "EVENT" && json[1].is_string() && json[2].is_object() {
                            let subscription_id = json[1].as_str().unwrap();
                            let event: Event = serde_json::from_value(json[2].clone())?;
                            // Verify the event's signature
                            if verify_event(&event) {
                                // If the event is valid, add it to the appropriate subscription's event list
                                if let Some(events) = self.subscriptions.get_mut(subscription_id) {
                                    events.push(event);
                                }
                            }
                        }
                    }
                    tokio_tungstenite::tungstenite::Message::Binary(data) => {
                        println!("Received binary data: {:?}", data);
                    }
                    _ => {}
                }
            }
        }
        Ok(())
    }

    /// Retrieves the list of events for a given subscription ID.
    #[allow(dead_code)]
    pub fn get_events(&self, subscription_id: &str) -> Option<&Vec<Event>> {
        self.subscriptions.get(subscription_id)
    }
}
