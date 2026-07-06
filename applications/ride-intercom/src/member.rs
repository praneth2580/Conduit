use conduit_sdk::{LocationPayload, NodeId};

/// GPS position in microdegrees (degrees × 1_000_000).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RidePosition {
  pub latitude_microdeg: i32,
  pub longitude_microdeg: i32,
  pub altitude_m: i16,
  pub accuracy_m: u16,
}

impl RidePosition {
  pub fn new(latitude_microdeg: i32, longitude_microdeg: i32) -> Self {
    Self {
      latitude_microdeg,
      longitude_microdeg,
      altitude_m: 0,
      accuracy_m: 10,
    }
  }

  pub fn to_location_payload(self) -> LocationPayload {
    LocationPayload {
      latitude_microdeg: self.latitude_microdeg,
      longitude_microdeg: self.longitude_microdeg,
      altitude_m: self.altitude_m,
      accuracy_m: self.accuracy_m,
    }
  }
}

/// A rider in the active group.
#[derive(Debug, Clone, PartialEq)]
pub struct RideMember {
  pub node_id: NodeId,
  pub name: String,
  pub last_location: Option<LocationPayload>,
}
