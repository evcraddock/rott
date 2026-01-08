//! Sync protocol message types
//!
//! Messages exchanged with automerge-repo-sync-server using CBOR encoding.

use serde::{Deserialize, Serialize};

use crate::document_id::DocumentId;

/// Peer ID for identifying this client
pub type PeerId = String;

/// Protocol version
pub const PROTOCOL_V1: &str = "1";

/// Peer metadata
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PeerMetadata {
    #[serde(default)]
    pub storage_id: Option<String>,
    #[serde(default)]
    pub is_ephemeral: bool,
}

/// Messages sent to the sync server
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum ClientMessage {
    /// Join/handshake message
    #[serde(rename = "join")]
    Join {
        #[serde(rename = "senderId")]
        sender_id: PeerId,
        #[serde(rename = "peerMetadata")]
        peer_metadata: PeerMetadata,
        #[serde(rename = "supportedProtocolVersions")]
        supported_protocol_versions: Vec<String>,
    },

    /// Sync message containing Automerge sync data
    #[serde(rename = "sync")]
    Sync {
        #[serde(rename = "senderId")]
        sender_id: PeerId,
        #[serde(rename = "targetId")]
        target_id: PeerId,
        #[serde(rename = "documentId")]
        document_id: String,
        /// Automerge sync message bytes
        #[serde(with = "serde_bytes")]
        data: Vec<u8>,
    },

    /// Request a document
    #[serde(rename = "request")]
    Request {
        #[serde(rename = "senderId")]
        sender_id: PeerId,
        #[serde(rename = "targetId")]
        target_id: PeerId,
        #[serde(rename = "documentId")]
        document_id: String,
        /// Automerge sync message bytes
        #[serde(with = "serde_bytes")]
        data: Vec<u8>,
    },
}

/// Messages received from the sync server
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "camelCase")]
pub enum ServerMessage {
    /// Peer handshake response
    #[serde(rename = "peer")]
    Peer {
        #[serde(rename = "senderId")]
        sender_id: PeerId,
        #[serde(rename = "targetId")]
        target_id: PeerId,
        #[serde(rename = "peerMetadata")]
        peer_metadata: PeerMetadata,
        #[serde(rename = "selectedProtocolVersion")]
        selected_protocol_version: String,
    },

    /// Sync message from server
    #[serde(rename = "sync")]
    Sync {
        #[serde(rename = "senderId")]
        sender_id: PeerId,
        #[serde(rename = "targetId")]
        target_id: PeerId,
        #[serde(rename = "documentId")]
        document_id: String,
        /// Automerge sync message bytes
        #[serde(with = "serde_bytes")]
        data: Vec<u8>,
    },

    /// Error from server
    #[serde(rename = "error")]
    Error {
        #[serde(rename = "senderId")]
        sender_id: PeerId,
        #[serde(rename = "targetId")]
        target_id: PeerId,
        message: String,
    },

    /// Document unavailable
    #[serde(rename = "doc-unavailable")]
    DocUnavailable {
        #[serde(rename = "senderId")]
        sender_id: PeerId,
        #[serde(rename = "targetId")]
        target_id: PeerId,
        #[serde(rename = "documentId")]
        document_id: String,
    },
}

impl ClientMessage {
    /// Create a join message
    pub fn join(sender_id: &str) -> Self {
        ClientMessage::Join {
            sender_id: sender_id.to_string(),
            peer_metadata: PeerMetadata::default(),
            supported_protocol_versions: vec![PROTOCOL_V1.to_string()],
        }
    }

    /// Create a sync message
    pub fn sync(sender_id: &str, target_id: &str, doc_id: &DocumentId, data: Vec<u8>) -> Self {
        ClientMessage::Sync {
            sender_id: sender_id.to_string(),
            target_id: target_id.to_string(),
            document_id: doc_id.to_bs58check(),
            data,
        }
    }

    /// Create a request message
    pub fn request(sender_id: &str, target_id: &str, doc_id: &DocumentId, data: Vec<u8>) -> Self {
        ClientMessage::Request {
            sender_id: sender_id.to_string(),
            target_id: target_id.to_string(),
            document_id: doc_id.to_bs58check(),
            data,
        }
    }

    /// Encode message to CBOR bytes
    pub fn encode(&self) -> Vec<u8> {
        let mut bytes = Vec::new();
        ciborium::into_writer(self, &mut bytes).expect("CBOR encoding failed");
        bytes
    }
}

impl ServerMessage {
    /// Decode message from CBOR bytes
    pub fn decode(bytes: &[u8]) -> Result<Self, ciborium::de::Error<std::io::Error>> {
        ciborium::from_reader(bytes)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_join_message_encoding() {
        let msg = ClientMessage::join("peer-123");
        let bytes = msg.encode();

        // Should be non-empty CBOR
        assert!(!bytes.is_empty());
    }

    #[test]
    fn test_sync_message_encoding() {
        let doc_id = DocumentId::new();
        let msg = ClientMessage::sync("peer-1", "peer-2", &doc_id, vec![1, 2, 3, 4]);
        let bytes = msg.encode();

        assert!(!bytes.is_empty());
    }

    #[test]
    fn test_server_message_decoding() {
        // Create a peer message manually in CBOR
        let msg = ServerMessage::Peer {
            sender_id: "server".to_string(),
            target_id: "client".to_string(),
            peer_metadata: PeerMetadata::default(),
            selected_protocol_version: "1".to_string(),
        };

        // Encode and decode
        let mut bytes = Vec::new();
        ciborium::into_writer(&msg, &mut bytes).unwrap();
        let decoded = ServerMessage::decode(&bytes).unwrap();

        match decoded {
            ServerMessage::Peer { sender_id, .. } => {
                assert_eq!(sender_id, "server");
            }
            _ => panic!("Expected Peer message"),
        }
    }
}
