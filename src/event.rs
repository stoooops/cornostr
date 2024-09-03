use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/*
## Events and signatures

Each user has a keypair. Signatures, public key, and encodings are done according to the [Schnorr signatures standard for the curve `secp256k1`](https://bips.xyz/340).


*/

/// Represents a Nostr event.
///
/// From NIP-01:
///
/// The only object type that exists is the `event`, which has the following format on the wire:
/// ```jsonc
/// {
///   "id": <32-bytes lowercase hex-encoded sha256 of the serialized event data>,
///   "pubkey": <32-bytes lowercase hex-encoded public key of the event creator>,
///   "created_at": <unix timestamp in seconds>,
///   "kind": <integer between 0 and 65535>,
///   "tags": [
///     [<arbitrary string>...],
///     // ...
///   ],
///   "content": <arbitrary string>,
///   "sig": <64-bytes lowercase hex of the signature of the sha256 hash of the serialized event data, which is the same as the "id" field>
/// }
/// ```
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Event {
    /// 32-bytes lowercase hex-encoded sha256 of the serialized event data
    pub id: String,
    /// 32-bytes lowercase hex-encoded public key of the event creator
    pub pubkey: String,
    /// unix timestamp in seconds
    pub created_at: u64,
    /// integer between 0 and 65535
    pub kind: u32,
    /// Arbitrary strings.
    pub tags: Vec<Vec<String>>,
    /// Arbitrary string.
    pub content: String,
    /// 64-bytes lowercase hex of the signature of the sha256 hash of the serialized event data, which is the same as the "id" field
    pub sig: String,
}

/// Calculates the ID for a Nostr event.
///
/// To obtain the `event.id`, we `sha256` the serialized event. The serialization is done over the UTF-8
/// JSON-serialized string (which is described below) of the following structure:
/// ```jsonc
/// [
///   0,
///   <pubkey, as a lowercase hex string>,
///   <created_at, as a number>,
///   <kind, as a number>,
///   <tags, as an array of arrays of non-null strings>,
///   <content, as a string>
/// ]
/// ```
/// To prevent implementation differences from creating a different event ID for the same event, the following rules
/// MUST be followed while serializing:
///
/// - UTF-8 should be used for encoding.
/// - Whitespace, line breaks or other unnecessary formatting should not be included in the output JSON.
/// - The following characters in the content field must be escaped as shown, and all other characters must be
///   included verbatim:
///   - A line break (`0x0A`), use `\n`
///   - A double quote (`0x22`), use `\"`
///   - A backslash (`0x5C`), use `\\`
///   - A carriage return (`0x0D`), use `\r`
///   - A tab character (`0x09`), use `\t`
///   - A backspace, (`0x08`), use `\b`
///   - A form feed, (`0x0C`), use `\f`
pub fn calculate_event_id(event: &Event) -> String {
    let serialized = serialize_event(event);
    let mut hasher = Sha256::new();
    hasher.update(serialized);
    hex::encode(hasher.finalize())
}

/// Serializes an event for ID calculation and signing.
pub fn serialize_event(event: &Event) -> Vec<u8> {
    let serialized = format!(
        "[0,\"{}\",{},{},{},{}]",
        event.pubkey,
        event.created_at,
        event.kind,
        serde_json::to_string(&event.tags).unwrap(),
        serde_json::to_string(&event.content).unwrap()
    );
    serialized.into_bytes()
}

#[cfg(test)]
mod tests {
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
    fn test_event_id_calculation() {
        let event = test_event();

        let id = calculate_event_id(&event);
        assert_eq!(id.len(), 64);
        assert!(hex::decode(&id).is_ok());
    }

    #[test]
    fn test_calculate_event_id() {
        let event = test_event();

        assert_eq!(event.id, calculate_event_id(&event));
    }

    #[test]
    fn test_serialize_event() {
        let event = test_event();

        let serialized = serialize_event(&event);
        let tags_serialized = serde_json::to_string(&event.tags).unwrap();
        let content_serialized = serde_json::to_string(&event.content).unwrap();
        let expected = format!(
            "[0,\"{}\",{},{},{},{}]",
            event.pubkey, event.created_at, event.kind, tags_serialized, content_serialized
        );
        assert_eq!(String::from_utf8(serialized).unwrap(), expected);
    }

    #[test]
    fn test_serialize_event_with_escape_characters() {
        let test_cases = vec![
            (
                "Line\nBreak",
                "[0,\"pubkey\",1234567890,1,[],\"Line\\nBreak\"]",
            ),
            (
                "Double\"Quote",
                "[0,\"pubkey\",1234567890,1,[],\"Double\\\"Quote\"]",
            ),
            (
                "Back\\slash",
                "[0,\"pubkey\",1234567890,1,[],\"Back\\\\slash\"]",
            ),
            (
                "Carriage\rReturn",
                "[0,\"pubkey\",1234567890,1,[],\"Carriage\\rReturn\"]",
            ),
            (
                "Tab\tCharacter",
                "[0,\"pubkey\",1234567890,1,[],\"Tab\\tCharacter\"]",
            ),
            (
                "Back\x08space",
                "[0,\"pubkey\",1234567890,1,[],\"Back\\bspace\"]",
            ),
            (
                "Form\x0CFeed",
                "[0,\"pubkey\",1234567890,1,[],\"Form\\fFeed\"]",
            ),
        ];

        for (content, expected) in test_cases {
            let event = Event {
                id: String::new(),
                pubkey: "pubkey".to_string(),
                created_at: 1234567890,
                kind: 1,
                tags: vec![],
                content: content.to_string(),
                sig: String::new(),
            };

            let serialized = serialize_event(&event);
            assert_eq!(String::from_utf8(serialized).unwrap(), expected);
        }
    }
}
