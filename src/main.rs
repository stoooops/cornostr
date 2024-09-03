use std::env;

mod client;
mod crypto;
mod event;
mod relay;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 3 {
        eprintln!("Usage: {} <client|relay> <address>", args[0]);
        std::process::exit(1);
    }

    let mode = &args[1];
    let address = &args[2];

    match mode.as_str() {
        "client" => {
            let mut client = client::Client::new();
            client.generate_keypair();
            client.connect(address).await?;

            let subscription_id = "my_subscription";
            let filter = r#"{"kinds": [1], "limit": 10}"#;
            client.subscribe(subscription_id, filter).await?;

            client.receive_events().await?;
        }
        "relay" => {
            let relay = relay::Relay::new();
            relay.run(address).await?;
        }
        _ => {
            eprintln!("Invalid mode. Use 'client' or 'relay'.");
            std::process::exit(1);
        }
    }

    Ok(())
}
