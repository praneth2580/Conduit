mod args;
mod commands;
mod display;
mod error;
mod opts;

pub use args::Cli;
pub use error::{CliError, CliResult};

use args::Commands;

pub fn run(cli: Cli) -> CliResult<()> {
  match cli.command {
    Commands::Version => commands::version::run(),
    Commands::Config { command } => commands::config::run(command),
    Commands::Node { command } => commands::node::run(command),
    Commands::Run(opts) => commands::run::run(opts),
    Commands::Send { command } => commands::send::run(command),
    Commands::Sim { command } => commands::sim::run(command),
    Commands::Ride { command } => commands::ride::run(command),
  }
}
