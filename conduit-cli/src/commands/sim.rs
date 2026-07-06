use crate::args::SimCommands;
use crate::display::{print_ride_event, print_sdk_event};
use crate::opts::sdk_config_seeded;
use conduit_sdk::{
  Conduit, DiscoveredPeer, DriverKind, PeerEndpoint, SimBusHandle,
};
use ride_intercom::{RideIntercom, RideIntercomConfig, RidePosition};

pub fn run(command: SimCommands) -> crate::CliResult<()> {
  match command {
    SimCommands::Exchange { message } => exchange(&message),
    SimCommands::Ride { group } => ride_smoke(&group),
  }
}

fn exchange(message: &str) -> crate::CliResult<()> {
  let bus = SimBusHandle::new();
  let mut alice = node("alice", 1, &bus);
  let mut bob = node("bob", 2, &bus);

  alice.join_network()?;
  bob.join_network()?;
  link(&mut alice, &mut bob)?;

  println!("alice -> bob: {message}");
  alice.send_broadcast(message)?;
  alice.tick()?;
  for event in bob.tick()? {
    print_sdk_event(&event);
  }

  alice.leave_network()?;
  bob.leave_network()?;
  Ok(())
}

fn ride_smoke(group: &str) -> crate::CliResult<()> {
  let bus = SimBusHandle::new();
  let mut a = ride("alex", 3, group, &bus);
  let mut b = ride("blake", 4, group, &bus);

  a.start()?;
  b.start()?;
  RideIntercom::link_riders(&mut a, &mut b)?;

  a.set_position(RidePosition::new(51_507_400, -0_127_800));
  a.share_location()?;
  a.trigger_sos("simulated breakdown")?;

  a.tick()?;
  let events = b.tick()?;
  for event in &events {
    print_ride_event(event);
  }

  let got_location = events.iter().any(|e| {
    matches!(
      e,
      ride_intercom::RideEvent::MemberLocationUpdated { .. }
    )
  });
  let got_sos = events
    .iter()
    .any(|e| matches!(e, ride_intercom::RideEvent::SosReceived { .. }));

  if !got_location || !got_sos {
    return Err(crate::CliError::Message(
      "ride simulation did not deliver location and SOS".into(),
    ));
  }

  println!("ride simulation ok");
  a.stop()?;
  b.stop()?;
  Ok(())
}

fn node(name: &str, seed: u8, bus: &SimBusHandle) -> Conduit {
  let mut config = sdk_config_seeded(name, seed);
  config.sim_bus = Some(bus.clone());
  Conduit::initialize(config).unwrap()
}

fn ride(name: &str, seed: u8, group: &str, bus: &SimBusHandle) -> RideIntercom {
  let sdk = {
    let mut c = sdk_config_seeded(name, seed);
    c.sim_bus = Some(bus.clone());
    c
  };
  let config = RideIntercomConfig::builder()
    .sdk(sdk)
    .rider_name(name)
    .group_name(group)
    .location_share_interval_ms(0)
    .push_to_talk(false)
    .build();
  RideIntercom::new(config).unwrap()
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
