//! Group protocol helpers for ride intercom sessions.

pub const PREFIX: &str = "CONDUIT_RIDE";

pub fn join_message(group: &str, rider: &str) -> String {
  format!("{PREFIX}:JOIN:{group}:{rider}")
}

pub fn leave_message(group: &str, rider: &str) -> String {
  format!("{PREFIX}:LEAVE:{group}:{rider}")
}

/// Parsed ride-group control message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GroupMessage {
  Join { group: String, rider: String },
  Leave { group: String, rider: String },
}

pub fn parse(content: &str) -> Option<GroupMessage> {
  let rest = content.strip_prefix(PREFIX)?;
  let parts: Vec<&str> = rest.split(':').collect();
  match parts.as_slice() {
    ["", "JOIN", group, rider] => Some(GroupMessage::Join {
      group: (*group).into(),
      rider: (*rider).into(),
    }),
    ["", "LEAVE", group, rider] => Some(GroupMessage::Leave {
      group: (*group).into(),
      rider: (*rider).into(),
    }),
    _ => None,
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn round_trip_join() {
    let msg = join_message("sunday-ride", "alex");
    assert_eq!(
      parse(&msg),
      Some(GroupMessage::Join {
        group: "sunday-ride".into(),
        rider: "alex".into(),
      })
    );
  }
}
