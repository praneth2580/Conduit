use crate::version::ProtocolVersion;

/// Maximum payload size before fragmentation (transport layer handles splitting).
pub const MAX_PAYLOAD_SIZE: usize = 1_400;

/// Maximum total packet size including header.
pub const MAX_PACKET_SIZE: usize = MAX_PAYLOAD_SIZE + PACKET_HEADER_SIZE;

/// Serialized size of the common packet header.
pub const PACKET_HEADER_SIZE: usize = 58;

/// Default time-to-live for routed packets.
pub const DEFAULT_TTL: u8 = 16;

/// Maximum TTL allowed on the wire.
pub const MAX_TTL: u8 = 64;

/// Default heartbeat interval in milliseconds.
pub const DEFAULT_HEARTBEAT_INTERVAL_MS: u64 = 5_000;

/// Neighbor timeout — consider a peer gone after this many milliseconds without contact.
pub const DEFAULT_NEIGHBOR_TIMEOUT_MS: u64 = 15_000;

/// Protocol version embedded in every packet.
pub const PROTOCOL_VERSION: ProtocolVersion = ProtocolVersion::CURRENT;

/// Magic bytes identifying a Conduit frame on the wire.
pub const FRAME_MAGIC: [u8; 4] = *b"CDIT";

/// Size of the frame magic prefix.
pub const FRAME_MAGIC_SIZE: usize = FRAME_MAGIC.len();
