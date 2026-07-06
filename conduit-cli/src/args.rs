use clap::Parser;
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(
  name = "conduit",
  version,
  about = "Conduit mesh communication platform CLI",
  long_about = "Operate, debug, and simulate Conduit mesh nodes."
)]
pub struct Cli {
  #[command(subcommand)]
  pub command: Commands,
}

#[derive(Debug, clap::Subcommand)]
pub enum Commands {
  /// Show build and protocol version information.
  Version,
  /// Manage Conduit configuration files.
  Config {
    #[command(subcommand)]
    command: ConfigCommands,
  },
  /// Node identity utilities.
  Node {
    #[command(subcommand)]
    command: NodeCommands,
  },
  /// Run a Conduit node and print mesh events.
  Run(RunOpts),
  /// Send a one-shot packet from a node.
  Send {
    #[command(subcommand)]
    command: SendCommands,
  },
  /// Local multi-node simulations (no network required).
  Sim {
    #[command(subcommand)]
    command: SimCommands,
  },
  /// Ride intercom application commands.
  Ride {
    #[command(subcommand)]
    command: RideCommands,
  },
}

#[derive(Debug, clap::Args)]
pub struct RunOpts {
  /// Human-readable node name.
  #[arg(short, long, default_value = "conduit-node")]
  pub name: String,
  /// Use in-memory simulation instead of UDP discovery.
  #[arg(long)]
  pub simulation: bool,
  /// Log level: trace, debug, info, warn, error.
  #[arg(long, default_value = "info")]
  pub log_level: String,
  /// Milliseconds between mesh ticks.
  #[arg(long, default_value_t = 1_000)]
  pub tick_ms: u64,
  /// Stop after N ticks (0 = run until interrupted).
  #[arg(long, default_value_t = 0)]
  pub ticks: u64,
}

#[derive(Debug, clap::Subcommand)]
pub enum ConfigCommands {
  /// Write a default configuration file.
  Init {
    /// Output path.
    #[arg(short, long, default_value = "conduit.json")]
    output: PathBuf,
  },
  /// Validate a configuration file.
  Validate {
    /// Config file path.
    #[arg(short, long, default_value = "conduit.json")]
    file: PathBuf,
  },
}

#[derive(Debug, clap::Subcommand)]
pub enum NodeCommands {
  /// Generate a random node ID.
  Id {
    /// Print only the first 8 hex chars.
    #[arg(long)]
    short: bool,
  },
}

#[derive(Debug, clap::Args)]
pub struct NodeNameOpt {
  #[arg(short, long, default_value = "conduit-node")]
  pub name: String,
}

#[derive(Debug, clap::Subcommand)]
pub enum SendCommands {
  /// Broadcast a text message.
  Message {
    #[command(flatten)]
    node: NodeNameOpt,
    /// Message body.
    #[arg(short, long)]
    text: String,
    /// Use simulation transport.
    #[arg(long)]
    simulation: bool,
    /// Ticks to run after sending (for simulation delivery).
    #[arg(long, default_value_t = 2)]
    ticks: u64,
  },
  /// Broadcast a GPS location.
  Location {
    #[command(flatten)]
    node: NodeNameOpt,
    /// Latitude in microdegrees (degrees × 1_000_000).
    #[arg(long, allow_hyphen_values = true)]
    lat: i32,
    /// Longitude in microdegrees.
    #[arg(long, allow_hyphen_values = true)]
    lon: i32,
    #[arg(long, default_value_t = 0, allow_hyphen_values = true)]
    alt: i16,
    #[arg(long, default_value_t = 10)]
    accuracy: u16,
    #[arg(long)]
    simulation: bool,
    #[arg(long, default_value_t = 2)]
    ticks: u64,
  },
  /// Broadcast an emergency / SOS signal.
  Sos {
    #[command(flatten)]
    node: NodeNameOpt,
    #[arg(short, long)]
    message: String,
    #[arg(long)]
    simulation: bool,
    #[arg(long, default_value_t = 2)]
    ticks: u64,
  },
}

#[derive(Debug, clap::Subcommand)]
pub enum SimCommands {
  /// Link two simulated nodes and exchange a message.
  Exchange {
    #[arg(short, long, default_value = "hello from conduit")]
    message: String,
  },
  /// Two-rider location and SOS smoke test.
  Ride {
    #[arg(long, default_value = "sunday-ride")]
    group: String,
  },
}

#[derive(Debug, clap::Subcommand)]
pub enum RideCommands {
  /// Start a ride intercom session.
  Run {
    #[arg(short, long)]
    rider: String,
    #[arg(short, long, default_value = "default-group")]
    group: String,
    #[arg(long)]
    simulation: bool,
    #[arg(long, default_value_t = 1_000)]
    tick_ms: u64,
    #[arg(long, default_value_t = 5)]
    ticks: u64,
    /// Optional latitude microdegrees for GPS sharing demo.
    #[arg(long, allow_hyphen_values = true)]
    lat: Option<i32>,
    /// Optional longitude microdegrees for GPS sharing demo.
    #[arg(long, allow_hyphen_values = true)]
    lon: Option<i32>,
  },
}

impl Cli {
  pub fn parse() -> Self {
    <Self as Parser>::parse()
  }
}
