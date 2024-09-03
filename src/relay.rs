use futures_util::{SinkExt, StreamExt};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::{mpsc, Mutex};
use tokio_tungstenite::{accept_async, tungstenite::Message};

use crate::event::Event;

struct Client {
    tx: mpsc::Sender<Message>,
    subscriptions: HashMap<String, ()>,
}

pub struct Relay {
    events: Arc<Mutex<Vec<Event>>>,
    clients: Arc<Mutex<HashMap<usize, Client>>>,
    next_client_id: AtomicUsize,
}

impl Relay {
    pub fn new() -> Self {
        Relay {
            events: Arc::new(Mutex::new(Vec::new())),
            clients: Arc::new(Mutex::new(HashMap::new())),
            next_client_id: AtomicUsize::new(0),
        }
    }

    pub async fn run(&self, addr: &str) -> Result<(), Box<dyn std::error::Error>> {
        let listener = TcpListener::bind(addr).await?;
        println!("Relay listening on: {}", addr);

        while let Ok((stream, _)) = listener.accept().await {
            let ws_stream = accept_async(stream).await?;
            let (write, read) = ws_stream.split();
            let (tx, rx) = mpsc::channel(100);
            let client_id = self.next_client_id.fetch_add(1, Ordering::SeqCst);

            self.clients.lock().await.insert(
                client_id,
                Client {
                    tx: tx.clone(),
                    subscriptions: HashMap::new(),
                },
            );

            let clients = Arc::clone(&self.clients);
            let events = Arc::clone(&self.events);

            tokio::spawn(Self::client_writer(write, rx));
            tokio::spawn(Self::client_reader(client_id, read, clients, events));
        }

        Ok(())
    }

    async fn client_writer(
        mut write: futures_util::stream::SplitSink<
            tokio_tungstenite::WebSocketStream<tokio::net::TcpStream>,
            Message,
        >,
        mut rx: mpsc::Receiver<Message>,
    ) {
        while let Some(message) = rx.recv().await {
            if let Err(e) = write.send(message).await {
                eprintln!("Failed to send message: {:?}", e);
                break;
            }
        }
    }

    async fn client_reader(
        client_id: usize,
        mut read: futures_util::stream::SplitStream<
            tokio_tungstenite::WebSocketStream<tokio::net::TcpStream>,
        >,
        clients: Arc<Mutex<HashMap<usize, Client>>>,
        events: Arc<Mutex<Vec<Event>>>,
    ) {
        while let Some(Ok(message)) = read.next().await {
            if let Ok(text) = message.into_text() {
                if let Ok(json) = serde_json::from_str::<Value>(&text) {
                    Self::handle_message(client_id, json, &clients, &events).await;
                }
            }
        }
        clients.lock().await.remove(&client_id);
    }

    async fn handle_message(
        client_id: usize,
        json: Value,
        clients: &Arc<Mutex<HashMap<usize, Client>>>,
        events: &Arc<Mutex<Vec<Event>>>,
    ) {
        match json[0].as_str() {
            Some("EVENT") => Self::handle_event(json, events, clients).await,
            Some("REQ") => Self::handle_req(client_id, json, clients).await,
            Some("CLOSE") => Self::handle_close(client_id, json, clients).await,
            _ => println!("Unknown message type"),
        }
    }

    async fn handle_event(
        json: Value,
        events: &Arc<Mutex<Vec<Event>>>,
        clients: &Arc<Mutex<HashMap<usize, Client>>>,
    ) {
        if let Ok(event) = serde_json::from_value::<Event>(json[1].clone()) {
            events.lock().await.push(event.clone());
            let clients = clients.lock().await;
            for client in clients.values() {
                for subscription_id in client.subscriptions.keys() {
                    if event_matches_subscription(&event, subscription_id) {
                        let message = serde_json::json!(["EVENT", subscription_id, event]);
                        let _ = client
                            .tx
                            .send(Message::Text(serde_json::to_string(&message).unwrap()))
                            .await;
                        break; // Send the event only once per client, even if it matches multiple subscriptions
                    }
                }
            }
        }
    }

    async fn handle_req(
        client_id: usize,
        json: Value,
        clients: &Arc<Mutex<HashMap<usize, Client>>>,
    ) {
        if let (Some(subscription_id), Some(_filter)) = (json[1].as_str(), json[2].as_object()) {
            let mut clients = clients.lock().await;
            if let Some(client) = clients.get_mut(&client_id) {
                client.subscriptions.insert(subscription_id.to_string(), ());
            }
        }
    }

    async fn handle_close(
        client_id: usize,
        json: Value,
        clients: &Arc<Mutex<HashMap<usize, Client>>>,
    ) {
        if let Some(subscription_id) = json[1].as_str() {
            let mut clients = clients.lock().await;
            if let Some(client) = clients.get_mut(&client_id) {
                client.subscriptions.remove(subscription_id);
            }
        }
    }
}

// You need to implement this function based on your subscription filter logic
fn event_matches_subscription(_event: &Event, _subscription_id: &str) -> bool {
    // Implement your filter logic here
    // For now, we'll just return true to send all events to all subscriptions
    true
}

impl Default for Relay {
    fn default() -> Self {
        Self::new()
    }
}
