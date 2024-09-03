use cornostr::client::Client;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <relay_address>", args[0]);
        std::process::exit(1);
    }

    let relay_addr = &args[1];
    let mut client = Client::new();

    // Generate a keypair for the client
    println!("Generating keypair...");
    client.generate_keypair();

    // Connect to the relay
    println!("Connecting to relay...");
    client.connect(relay_addr).await?;

    // Example: Create a subscription
    let subscription_id = "my_subscription";
    let filter = r#"{"kinds": [1], "limit": 10}"#;
    println!("Subscribing to events...");
    client.subscribe(subscription_id, filter).await?;

    // Start receiving events
    println!("Receiving events...");
    client.receive_events().await?;

    // You can add more functionality here, such as publishing events or processing received events

    Ok(())
}
