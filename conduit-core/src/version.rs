use crate::error::{ConduitError, Result};
use crate::serialization::{ByteReader, ByteWriter};
use serde::{Deserialize, Serialize};
use std::fmt;

/// Protocol version carried in every packet header.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ProtocolVersion {
    pub major: u8,
    pub minor: u8,
}

impl ProtocolVersion {
    pub const CURRENT: Self = Self { major: 0, minor: 1 };

    pub const fn new(major: u8, minor: u8) -> Self {
        Self { major, minor }
    }

    pub fn is_compatible_with(&self, other: &Self) -> bool {
        self.major == other.major
    }

    pub fn encode(&self, writer: &mut ByteWriter) {
        writer.write_u8(self.major);
        writer.write_u8(self.minor);
    }

    pub fn decode(reader: &mut ByteReader) -> Result<Self> {
        let major = reader.read_u8()?;
        let minor = reader.read_u8()?;
        Ok(Self { major, minor })
    }

    pub fn from_str(s: &str) -> Result<Self> {
        let (major, minor) = s
            .split_once('.')
            .ok_or_else(|| ConduitError::Configuration(format!("invalid version: {s}")))?;
        Ok(Self {
            major: major
                .parse()
                .map_err(|_| ConduitError::Configuration(format!("invalid major: {major}")))?,
            minor: minor
                .parse()
                .map_err(|_| ConduitError::Configuration(format!("invalid minor: {minor}")))?,
        })
    }
}

impl fmt::Display for ProtocolVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}", self.major, self.minor)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_compatibility_requires_matching_major() {
        let a = ProtocolVersion::new(1, 0);
        let b = ProtocolVersion::new(1, 3);
        let c = ProtocolVersion::new(2, 0);
        assert!(a.is_compatible_with(&b));
        assert!(!a.is_compatible_with(&c));
    }

    #[test]
    fn version_round_trip() {
        let version = ProtocolVersion::CURRENT;
        let mut writer = ByteWriter::new();
        version.encode(&mut writer);
        let mut reader = ByteReader::new(writer.as_slice());
        assert_eq!(ProtocolVersion::decode(&mut reader).unwrap(), version);
    }
}
