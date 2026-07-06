use crate::args::RunOpts;
use crate::display::print_sdk_event;
use crate::opts::sdk_config;
use conduit_sdk::Conduit;
use std::thread;
use std::time::Duration;

pub fn run(opts: RunOpts) -> crate::CliResult<()> {
  let config = sdk_config(&opts.name, opts.simulation, &opts.log_level)?;
  let mut conduit = Conduit::initialize(config)?;
  conduit.join_network()?;
  println!(
    "node {} ({}) running ({})",
    conduit.node_id(),
    opts.name,
    if opts.simulation {
      "simulation"
    } else {
      "udp discovery"
    }
  );

  let mut count = 0u64;
  loop {
    for event in conduit.tick()? {
      print_sdk_event(&event);
    }
    count += 1;
    if opts.ticks > 0 && count >= opts.ticks {
      break;
    }
    thread::sleep(Duration::from_millis(opts.tick_ms));
  }

  conduit.leave_network()?;
  Ok(())
}
