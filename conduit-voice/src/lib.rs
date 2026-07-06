//! Conduit Voice Engine — capture, encode, packetize, jitter-buffer, decode, playback.
//!
//! Voice is just another packet type. The routing layer never inspects payload
//! contents — only the header's `PacketType::Voice` field.

mod codec;
mod config;
mod echo;
mod engine;
mod jitter;
mod noise;
mod packetizer;
mod pipeline;
mod vad;

pub use codec::{AudioCodec, LinearCodec};
pub use config::{VoiceConfig, DEFAULT_FRAME_SAMPLES, DEFAULT_FRAME_MS};
pub use engine::VoiceEngine;
pub use jitter::JitterBuffer;
pub use packetizer::VoicePacketizer;
pub use echo::{EchoCanceller, PassthroughEchoCanceller};
pub use noise::{NoiseSuppressor, PassthroughNoiseSuppressor};
pub use pipeline::VoicePipeline;
pub use vad::VoiceActivityDetector;

#[cfg(feature = "opus")]
pub use codec::OpusCodec;

pub use conduit_core::{NodeId, PacketType, Result};
pub use conduit_mesh::BROADCAST_NODE_ID;
pub use conduit_packets::{PacketBody, TypedPacket, VoicePayload};
