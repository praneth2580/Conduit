use crate::args::RideCommands;
use crate::display::print_ride_event;
use crate::opts::sdk_config_seeded;
use ride_intercom::{RideIntercom, RideIntercomConfig, RidePosition};
use std::thread;
use std::time::Duration;

pub fn run(command: RideCommands) -> crate::CliResult<()> {
  match command {
    RideCommands::Run {
      rider,
      group,
      simulation,
      tick_ms,
      ticks,
      lat,
      lon,
    } => ride_run(&rider, &group, simulation, tick_ms, ticks, lat, lon),
  }
}

fn ride_run(
  rider: &str,
  group: &str,
  simulation: bool,
  tick_ms: u64,
  ticks: u64,
  lat: Option<i32>,
  lon: Option<i32>,
) -> crate::CliResult<()> {
  let sdk = if simulation {
    sdk_config_seeded(rider, 1)
  } else {
    crate::opts::sdk_config(rider, false, "info")?
  };

  let config = RideIntercomConfig::builder()
    .sdk(sdk)
    .rider_name(rider)
    .group_name(group)
    .simulation(simulation)
    .location_share_interval_ms(5_000)
    .build();

  let mut ride = RideIntercom::new(config)?;
  for event in ride.start()? {
    print_ride_event(&event);
  }

  if let (Some(lat), Some(lon)) = (lat, lon) {
    ride.set_position(RidePosition::new(lat, lon));
    ride.share_location()?;
    println!("shared location: {lat}, {lon}");
  }

  println!(
    "ride intercom active — rider={rider} group={group} node={}",
    ride.node_id()
  );

  for _ in 0..ticks {
    for event in ride.tick()? {
      print_ride_event(&event);
    }
    thread::sleep(Duration::from_millis(tick_ms));
  }

  for event in ride.stop()? {
    print_ride_event(&event);
  }
  Ok(())
}
