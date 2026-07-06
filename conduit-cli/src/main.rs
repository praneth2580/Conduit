fn main() {
  let cli = conduit_cli::Cli::parse();
  if let Err(err) = conduit_cli::run(cli) {
    eprintln!("error: {err}");
    std::process::exit(1);
  }
}
