use crate::args::SendCommands;
use crate::display::print_sdk_event;
use crate::opts::{sdk_config, sdk_config_seeded};
use conduit_sdk::{Conduit, DiscoveredPeer, DriverKind, EmergencyPayload, PeerEndpoint, SimBusHandle};
use conduit_sdk::{EmergencyKind};

pub fn run(command: SendCommands) -> crate::CliResult<()> {
  match command {
    SendCommands::Message {
      node,
      text,
      simulation,
      ticks,
    } => send_message(&node.name, &text, simulation, ticks),
    SendCommands::Location {
      node,
      lat,
      lon,
      alt,
      accuracy,
      simulation,
      ticks,
    } => send_location(&node.name, lat, lon, alt, accuracy, simulation, ticks),
    SendCommands::Sos {
      node,
      message,
      simulation,
      ticks,
    } => send_sos(&node.name, &message, simulation, ticks),
  }
}

fn send_message(name: &str, text: &str, simulation: bool, ticks: u64) -> crate::CliResult<()> {
  if simulation {
    return sim_send(name, text, ticks, |c, t| c.send_broadcast(t));
  }
  let config = sdk_config(name, false, "warn")?;
  let mut conduit = Conduit::initialize(config)?;
  conduit.join_network()?;
  conduit.send_broadcast(text)?;
  drain(conduit, ticks)
}

fn send_location(
  name: &str,
  lat: i32,
  lon: i32,
  alt: i16,
  accuracy: u16,
  simulation: bool,
  ticks: u64,
) -> crate::CliResult<()> {
  if simulation {
    return sim_send(name, "", ticks, |c, _| {
      c.send_location(lat, lon, alt, accuracy)
    });
  }
  let config = sdk_config(name, false, "warn")?;
  let mut conduit = Conduit::initialize(config)?;
  conduit.join_network()?;
  conduit.send_location(lat, lon, alt, accuracy)?;
  drain(conduit, ticks)
}

fn send_sos(name: &str, message: &str, simulation: bool, ticks: u64) -> crate::CliResult<()> {
  if simulation {
    return sim_send(name, message, ticks, |c, msg| {
      c.send_emergency(EmergencyPayload {
        kind: EmergencyKind::General,
        message: msg.into(),
      })
    });
  }
  let config = sdk_config(name, false, "warn")?;
  let mut conduit = Conduit::initialize(config)?;
  conduit.join_network()?;
  conduit.send_emergency(EmergencyPayload {
    kind: EmergencyKind::General,
    message: message.into(),
  })?;
  drain(conduit, ticks)
}

fn drain(mut conduit: Conduit, ticks: u64) -> crate::CliResult<()> {
  for _ in 0..ticks.max(1) {
    for event in conduit.tick()? {
      print_sdk_event(&event);
    }
  }
  conduit.leave_network()?;
  Ok(())
}

fn sim_send<F>(name: &str, payload: &str, ticks: u64, send: F) -> crate::CliResult<()>
where
  F: Fn(&mut Conduit, &str) -> conduit_sdk::Result<()>,
{
  let bus = SimBusHandle::new();
  let mut sender = node_on_bus(name, 1, &bus);
  let mut receiver = node_on_bus("receiver", 2, &bus);
  sender.join_network()?;
  receiver.join_network()?;
  link(&mut sender, &mut receiver)?;

  send(&mut sender, payload)?;
  for _ in 0..ticks.max(1) {
    sender.tick()?;
    for event in receiver.tick()? {
      print_sdk_event(&event);
    }
  }
  sender.leave_network()?;
  receiver.leave_network()?;
  Ok(())
}

fn node_on_bus(name: &str, seed: u8, bus: &SimBusHandle) -> Conduit {
  let mut config = sdk_config_seeded(name, seed);
  config.sim_bus = Some(bus.clone());
  Conduit::initialize(config).unwrap()
}

fn link(a: &mut Conduit, b: &mut Conduit) -> crate::CliResult<()> {
  a.connect_peer(peer_for(b, 2))?;
  b.connect_peer(peer_for(a, 1))?;
  Ok(())
}

fn peer_for(other: &Conduit, sim_id: u64) -> DiscoveredPeer {
  DiscoveredPeer::new(
    other.node_id(),
    other.config().node_name.clone(),
    other.config().capabilities,
    DriverKind::Mock,
    PeerEndpoint::Simulated { id: sim_id },
  )
}
