use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn version_prints_protocol() {
  Command::cargo_bin("conduit")
    .unwrap()
    .arg("version")
    .assert()
    .success()
    .stdout(predicate::str::contains("protocol 0.1"));
}

#[test]
fn node_id_generates_uuid() {
  Command::cargo_bin("conduit")
    .unwrap()
    .args(["node", "id"])
    .assert()
    .success();
}

#[test]
fn sim_exchange_delivers_message() {
  Command::cargo_bin("conduit")
    .unwrap()
    .args([
      "sim",
      "exchange",
      "--message",
      "cli test message",
    ])
    .assert()
    .success()
    .stdout(predicate::str::contains("cli test message"));
}

#[test]
fn sim_ride_smoke() {
  Command::cargo_bin("conduit")
    .unwrap()
    .args(["sim", "ride", "--group", "cli-ride"])
    .assert()
    .success()
    .stdout(predicate::str::contains("ride simulation ok"));
}

#[test]
fn send_message_simulation() {
  Command::cargo_bin("conduit")
    .unwrap()
    .args([
      "send",
      "message",
      "--simulation",
      "--text",
      "ping",
      "--name",
      "sender",
    ])
    .assert()
    .success()
    .stdout(predicate::str::contains("ping"));
}

#[test]
fn run_simulation_ticks() {
  Command::cargo_bin("conduit")
    .unwrap()
    .args([
      "run",
      "--simulation",
      "--name",
      "tick-node",
      "--ticks",
      "2",
      "--tick-ms",
      "10",
      "--log-level",
      "error",
    ])
    .assert()
    .success()
    .stdout(predicate::str::contains("tick-node"));
}

#[test]
fn ride_run_accepts_negative_longitude() {
  Command::cargo_bin("conduit")
    .unwrap()
    .args([
      "ride",
      "run",
      "--rider",
      "alex",
      "--group",
      "sunday-ride",
      "--simulation",
      "--lat",
      "51507400",
      "--lon",
      "-127800",
      "--ticks",
      "1",
      "--tick-ms",
      "10",
    ])
    .assert()
    .success()
    .stdout(predicate::str::contains("shared location: 51507400, -127800"));
}
