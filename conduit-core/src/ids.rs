use crate::error::{ConduitError, Result};
use crate::serialization::{ByteReader, ByteWriter};
use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

/// Unique identifier for a node in the mesh.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeId(pub [u8; 16]);

impl NodeId {
    pub fn random() -> Self {
        Self(*Uuid::new_v4().as_bytes())
    }

    pub fn from_bytes(bytes: [u8; 16]) -> Self {
        Self(bytes)
    }

    pub fn as_bytes(&self) -> &[u8; 16] {
        &self.0
    }

    pub fn encode(&self, writer: &mut ByteWriter) {
        writer.write_bytes(&self.0);
    }

    pub fn decode(reader: &mut ByteReader) -> Result<Self> {
        let bytes = reader.read_bytes(16)?;
        let mut id = [0u8; 16];
        id.copy_from_slice(&bytes);
        Ok(Self(id))
    }
}

impl Default for NodeId {
    fn default() -> Self {
        Self::random()
    }
}

impl fmt::Display for NodeId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", Uuid::from_bytes(self.0))
    }
}

/// Monotonically increasing sequence number for outbound packets.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct PacketSequence(pub u32);

impl PacketSequence {
    pub const ZERO: Self = Self(0);

    pub fn next(self) -> Self {
        Self(self.0.wrapping_add(1))
    }

    pub fn encode(&self, writer: &mut ByteWriter) {
        writer.write_u32(self.0);
    }

    pub fn decode(reader: &mut ByteReader) -> Result<Self> {
        Ok(Self(reader.read_u32()?))
    }
}

impl fmt::Display for PacketSequence {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Parse a node ID from a UUID string.
pub fn parse_node_id(s: &str) -> Result<NodeId> {
    let uuid = Uuid::parse_str(s)
        .map_err(|e| ConduitError::Configuration(format!("invalid node id: {e}")))?;
    Ok(NodeId(*uuid.as_bytes()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn node_id_round_trip() {
        let id = NodeId::random();
        let mut writer = ByteWriter::new();
        id.encode(&mut writer);
        let mut reader = ByteReader::new(writer.as_slice());
        assert_eq!(NodeId::decode(&mut reader).unwrap(), id);
    }

    #[test]
    fn sequence_wraps_on_overflow() {
        let seq = PacketSequence(u32::MAX);
        assert_eq!(seq.next(), PacketSequence(0));
    }
}
