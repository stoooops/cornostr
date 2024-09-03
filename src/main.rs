use clap::{Parser, Subcommand};
use cornostr::client::Client;
use cornostr::crypto::generate_keypair;
use cornostr::post::create_note;
use cornostr::relay::Relay;
use std::error::Error;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run as a Nostr client
    Client {
        /// Relay address to connect to
        #[clap(short, long, default_value = "wss://relay.damus.io")]
        relay: String,

        #[clap(subcommand)]
        action: ClientAction,
    },
    /// Run as a Nostr relay
    Relay {
        /// Address to run the relay on
        #[clap(short, long)]
        address: String,
    },
}

#[derive(Subcommand)]
enum ClientAction {
    /// Subscribe to events
    Subscribe {
        /// Subscription ID
        #[clap(short, long, default_value = "my_subscription")]
        subscription_id: String,

        /// JSON filter for the subscription
        #[clap(short, long, default_value = r#"{"kinds": [1], "limit": 10}"#)]
        filter: String,
    },
    /// Publish a message
    Publish {
        /// Message content to publish
        #[clap(short, long)]
        message: String,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Client { relay, action } => {
            let mut client = Client::new();
            client.generate_keypair();
            client.connect(relay).await?;

            match action {
                ClientAction::Subscribe {
                    subscription_id,
                    filter,
                } => {
                    client.subscribe(subscription_id, filter).await?;
                    client.receive_events().await?;
                }
                ClientAction::Publish { message } => {
                    let keypair = generate_keypair();
                    let mut event = create_note(&keypair, message);
                    client.publish_event(&mut event).await?;
                    println!("Message published successfully!");
                }
            }
        }
        Commands::Relay { address } => {
            let relay = Relay::new();
            relay.run(address).await?;
        }
    }

    Ok(())
}
