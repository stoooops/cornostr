use cornostr::relay::Relay;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage: {} <relay_address>", args[0]);
        std::process::exit(1);
    }

    let relay_addr = &args[1];
    let relay = Relay::new(relay_addr)?;
    relay.connect().await?;

    Ok(())
}
