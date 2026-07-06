use conduit_core::Packet;

/// Why a packet was dropped by the routing engine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RoutingDropReason {
  Duplicate,
  TtlExpired,
  NoRoute,
  LoopDetected,
  Congested,
  InvalidPacket,
}

/// Decision produced after routing logic runs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RoutingAction {
  /// Packet is addressed to this node.
  Deliver(Packet),
  /// Deliver locally and relay to next hops (e.g. broadcast).
  DeliverAndForward {
    deliver: Packet,
    forward: Packet,
    next_hops: Vec<conduit_core::NodeId>,
  },
  /// Forward a (possibly decremented) packet to one or more next hops.
  Forward {
    packet: Packet,
    next_hops: Vec<conduit_core::NodeId>,
  },
  /// Drop the packet.
  Drop(RoutingDropReason),
}
