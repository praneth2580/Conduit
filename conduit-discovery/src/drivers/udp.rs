use crate::beacon::{decode_beacon, encode_beacon};
use crate::driver::{
  DiscoveryAnnouncement, DiscoveryDriver, DiscoveryEvent, DiscoveryState, DriverKind,
  PeerEndpoint,
};
use crate::peer::DiscoveredPeer;
use conduit_core::error::{ConduitError, Result};
use std::collections::VecDeque;
use std::net::{IpAddr, Ipv4Addr, SocketAddr, UdpSocket};

/// LAN discovery via UDP broadcast beacons.
#[derive(Debug)]
pub struct UdpBroadcastDriver {
  port: u16,
  state: DiscoveryState,
  socket: Option<UdpSocket>,
  local: Option<DiscoveryAnnouncement>,
  pending: VecDeque<DiscoveryEvent>,
}

impl UdpBroadcastDriver {
  pub fn new(port: u16) -> Self {
    Self {
      port,
      state: DiscoveryState::Stopped,
      socket: None,
      local: None,
      pending: VecDeque::new(),
    }
  }

  fn bind_socket(port: u16) -> Result<UdpSocket> {
    use socket2::{Domain, Socket, Type};
    let addr = SocketAddr::from((Ipv4Addr::UNSPECIFIED, port));
    let socket = Socket::new(Domain::IPV4, Type::DGRAM, None)?;
    socket.set_reuse_address(true)?;
    #[cfg(unix)]
    socket.set_reuse_port(true).ok();
    socket.bind(&addr.into())?;
    socket.set_broadcast(true)?;
    socket.set_nonblocking(true)?;
    Ok(socket.into())
  }

  fn broadcast_targets(port: u16) -> Vec<SocketAddr> {
    let mut targets = vec![SocketAddr::from((Ipv4Addr::BROADCAST, port))];
    if let Ok(interfaces) = if_addrs::get_if_addrs() {
      for iface in interfaces {
        if !iface.is_loopback() {
          if let if_addrs::IfAddr::V4(v4) = iface.addr {
            if let Some(broadcast) = v4.broadcast {
              targets.push(SocketAddr::new(IpAddr::V4(broadcast), port));
            }
          }
        }
      }
    }
    targets.sort();
    targets.dedup();
    targets
  }

  fn drain_incoming(&mut self) -> Result<()> {
    let socket = match self.socket.as_ref() {
      Some(s) => s,
      None => return Ok(()),
    };

    let mut buf = [0u8; 512];
    loop {
      match socket.recv_from(&mut buf) {
        Ok((len, addr)) => {
          let data = &buf[..len];
          let announcement = match decode_beacon(data) {
            Ok(a) => a,
            Err(_) => continue,
          };
          if Some(announcement.node_id) == self.local.as_ref().map(|l| l.node_id) {
            continue;
          }
          let peer = DiscoveredPeer::new(
            announcement.node_id,
            announcement.node_name,
            announcement.capabilities,
            DriverKind::UdpBroadcast,
            PeerEndpoint::Udp { addr },
          );
          self.pending.push_back(DiscoveryEvent::PeerFound(peer));
        }
        Err(e) if e.kind() == std::io::ErrorKind::WouldBlock => break,
        Err(e) => {
          return Err(ConduitError::Io(e));
        }
      }
    }
    Ok(())
  }
}

impl DiscoveryDriver for UdpBroadcastDriver {
  fn kind(&self) -> DriverKind {
    DriverKind::UdpBroadcast
  }

  fn state(&self) -> DiscoveryState {
    self.state.clone()
  }

  fn start(&mut self, announcement: &DiscoveryAnnouncement) -> Result<()> {
    if matches!(self.state, DiscoveryState::Running) {
      return Ok(());
    }
    self.socket = Some(Self::bind_socket(self.port)?);
    self.local = Some(announcement.clone());
    self.state = DiscoveryState::Running;
    self.announce(announcement)?;
    self
      .pending
      .push_back(DiscoveryEvent::DriverStarted {
        driver: DriverKind::UdpBroadcast,
      });
    Ok(())
  }

  fn stop(&mut self) -> Result<()> {
    self.state = DiscoveryState::Stopped;
    self.socket = None;
    self.local = None;
    self
      .pending
      .push_back(DiscoveryEvent::DriverStopped {
        driver: DriverKind::UdpBroadcast,
      });
    Ok(())
  }

  fn announce(&mut self, announcement: &DiscoveryAnnouncement) -> Result<()> {
    let socket = self
      .socket
      .as_ref()
      .ok_or_else(|| ConduitError::Configuration("udp driver is not running".into()))?;
    let payload = encode_beacon(announcement)?;
    let mut sent = false;
    for addr in Self::broadcast_targets(self.port) {
      match socket.send_to(&payload, addr) {
        Ok(_) => sent = true,
        Err(e) if e.kind() == std::io::ErrorKind::NetworkUnreachable => {}
        Err(e) => return Err(ConduitError::Io(e)),
      }
    }
    if !sent {
      return Err(ConduitError::Io(std::io::Error::new(
        std::io::ErrorKind::NetworkUnreachable,
        "no reachable broadcast targets",
      )));
    }
    self.local = Some(announcement.clone());
    Ok(())
  }

  fn poll(&mut self) -> Result<Vec<DiscoveryEvent>> {
    self.drain_incoming()?;
    Ok(self.pending.drain(..).collect())
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use conduit_core::NodeId;
  use std::thread;
  use std::time::Duration;

  fn free_port() -> u16 {
    let socket = UdpSocket::bind("0.0.0.0:0").unwrap();
    socket.local_addr().unwrap().port()
  }

  #[test]
  fn two_nodes_discover_each_other() {
    let port = free_port();

    let ann_a = DiscoveryAnnouncement {
      node_id: NodeId::random(),
      node_name: "node-a".into(),
      capabilities: 1,
    };
    let ann_b = DiscoveryAnnouncement {
      node_id: NodeId::random(),
      node_name: "node-b".into(),
      capabilities: 2,
    };

    let mut driver_a = UdpBroadcastDriver::new(port);
    let mut driver_b = UdpBroadcastDriver::new(port);
    driver_a.start(&ann_a).unwrap();
    driver_b.start(&ann_b).unwrap();

    driver_a.announce(&ann_a).unwrap();
    driver_b.announce(&ann_b).unwrap();

    thread::sleep(Duration::from_millis(100));

    let events_a: Vec<_> = driver_a.poll().unwrap();
    let events_b: Vec<_> = driver_b.poll().unwrap();

    assert!(events_a.iter().any(|e| matches!(e, DiscoveryEvent::PeerFound(p) if p.node_name == "node-b")));
    assert!(events_b.iter().any(|e| matches!(e, DiscoveryEvent::PeerFound(p) if p.node_name == "node-a")));
  }
}
