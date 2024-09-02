use cornostr::crypto::{generate_keypair, sign_event};
use cornostr::event::{calculate_event_id, Event};
use secp256k1::{Keypair, XOnlyPublicKey};
use std::time::{SystemTime, UNIX_EPOCH};

/// Creates a new text note Nostr event.
fn create_note(keypair: &Keypair, content: &str) -> Event {
    let (xonly_pubkey, _parity) = XOnlyPublicKey::from_keypair(keypair);

    let created_at = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs();

    let mut event = Event {
        id: String::new(),
        pubkey: hex::encode(xonly_pubkey.serialize()),
        created_at,
        kind: 1, // Text note
        tags: vec![],
        content: content.to_string(),
        sig: String::new(),
    };

    // Calculate the event ID
    event.id = calculate_event_id(&event);

    // Sign the event
    event.sig = sign_event(&event, keypair);

    event
}

fn main() {
    let keypair = generate_keypair();
    let event = create_note(&keypair, "Hello, Nostr!");
    println!("{:#?}", event);
}
