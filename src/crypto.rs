use rand::rngs::OsRng;
use secp256k1::{schnorr, Keypair, Message, Secp256k1, XOnlyPublicKey};

use crate::event::Event;

/// Generates a new secp256k1 keypair for use in Nostr.
pub fn generate_keypair() -> Keypair {
    // Create a new Secp256k1 context
    let secp = Secp256k1::new();

    // Generate a new keypair
    Keypair::new(&secp, &mut OsRng)
}

/// Signs a Nostr event using the provided secret key.
pub fn sign_event(event: &Event, keypair: &Keypair) -> String {
    // Create a message from the event ID
    let message = Message::from_digest_slice(&hex::decode(&event.id).unwrap()).unwrap();

    // Sign the message using Schnorr signature
    let signature = keypair.sign_schnorr(message);

    // Convert the signature to a hex-encoded string
    hex::encode(signature.as_ref())
}

/// Verifies the signature of a Nostr event.
pub fn verify_event(event: &Event) -> bool {
    let secp = Secp256k1::new();

    // Parse the public key
    let pubkey = match XOnlyPublicKey::from_slice(&hex::decode(&event.pubkey).unwrap()) {
        Ok(key) => key,
        Err(e) => {
            println!("Failed to parse public key: {:?}", e);
            return false;
        }
    };

    // Parse the signature
    let signature = match schnorr::Signature::from_slice(&hex::decode(&event.sig).unwrap()) {
        Ok(sig) => sig,
        Err(e) => {
            println!("Failed to parse schnorr signature: {:?}", e);
            return false;
        }
    };

    // Verify the signature
    let message = Message::from_digest_slice(&hex::decode(&event.id).unwrap()).unwrap();

    secp.verify_schnorr(&signature, &message, &pubkey).is_ok()
}

#[cfg(test)]
mod tests {

    use crate::event::calculate_event_id;

    use super::*;

    fn test_event() -> Event {
        Event {
            content: "Thank you!".to_string(),
            created_at: 1725316278,
            id: "4dc5e11a899e3a0496a31955a486a74800ba6d756e40fe0ceb67e3930bcb5dc6".to_string(),
            kind: 1,
            pubkey: "ae8ef5576370b5cb91d262cf0d31d5ce9f5ca26c3ad2d56d5c58f6023633e453".to_string(),
            sig: "44b4b5e4087504f7ca44bb72cb89c119e680f459739a476023a036075e93a5219dc21380fbda14af4c5008185c1fc86a08acb433fb7097eff175cc81174a345c".to_string(),
            tags: vec![
                vec!["e".to_string(),"f14669da001fc23052bbfa3e4124699a85dc14b3ecb65023a86ed16a317c1cc3".to_string(),"".to_string(),"root".to_string()],
                vec!["e".to_string(),"32928056b07792e9a92193720c67d3458351ea66fbc568cdc87be41a5faa92ce".to_string(),"wss://nos.lol".to_string(),"reply".to_string()],
                vec!["p".to_string(),"2f5759825226f1d57ef1652ba66114b2f938f7f5c50dc505708e5d8b31e4f3c9".to_string()]
                ]
            }
    }

    #[test]
    fn test_sign_event() {
        // Generate a new keypair
        let keypair = generate_keypair();

        // Extract the XOnlyPublicKey from the Keypair
        let (xonly_pubkey, _parity) = XOnlyPublicKey::from_keypair(&keypair);

        let mut event = Event {
            id: "".to_string(),
            pubkey: hex::encode(xonly_pubkey.serialize()),
            created_at: 1617932400,
            kind: 1,
            tags: vec![],
            content: "Hello, Nostr!".to_string(),
            sig: String::new(),
        };
        event.id = calculate_event_id(&event);

        event.sig = sign_event(&event, &keypair);
        assert_eq!(event.sig.len(), 128);
        assert!(hex::decode(&event.sig).is_ok());

        // now verify the signature
        assert!(verify_event(&event));
    }

    #[test]
    fn test_verify_event() {
        let event = test_event();

        println!("Event: {:#?}", event);
        assert!(verify_event(&event), "Event verification failed");

        // Test with invalid signature
        let mut invalid_event = event.clone();
        invalid_event.sig = hex::encode([0u8; 64]);
        assert!(!verify_event(&invalid_event));

        // Test with modified content
        let mut modified_event = event.clone();
        modified_event.content = "Modified content".to_string();
        modified_event.id = calculate_event_id(&modified_event);
        assert!(!verify_event(&modified_event));
    }
}
