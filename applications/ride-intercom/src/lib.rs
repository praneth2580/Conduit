//! Ride Intercom — motorcycle group communication on Conduit.
//!
//! Built entirely on [`conduit_sdk::Conduit`]; this crate never touches
//! routing, transport, or discovery internals.

mod app;
mod config;
mod events;
mod group;
mod member;

pub use app::RideIntercom;
pub use config::{RideIntercomConfig, RideIntercomConfigBuilder, DEFAULT_LOCATION_SHARE_INTERVAL_MS};
pub use events::{RideEvent, SharedPosition};
pub use member::{RideMember, RidePosition};

pub use conduit_sdk::{EmergencyKind, NodeId, Result};
