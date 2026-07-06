use crate::args::NodeCommands;
use crate::display::short_node_id;
use conduit_core::NodeId;

pub fn run(command: NodeCommands) -> crate::CliResult<()> {
  match command {
    NodeCommands::Id { short } => {
      let id = NodeId::random();
      if short {
        println!("{}", short_node_id(&id));
      } else {
        println!("{id}");
      }
      Ok(())
    }
  }
}
