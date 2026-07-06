use crate::types::{LogEntry, NetworkStats, PacketRecord, TickResponse};
use conduit_core::utils::unix_timestamp_ms;
use conduit_sdk::{Conduit, SdkConfig, SdkEvent, VoiceMode};
use std::ffi::{CStr, CString};
use std::os::raw::c_char;
use std::sync::{Mutex, OnceLock};

struct Runtime {
  conduit: Option<Conduit>,
  stats: NetworkStats,
  packet_log: Vec<PacketRecord>,
  event_log: Vec<LogEntry>,
  next_packet_id: u64,
  latency_samples: Vec<u64>,
}

impl Default for Runtime {
  fn default() -> Self {
    Self {
      conduit: None,
      stats: NetworkStats::default(),
      packet_log: Vec::new(),
      event_log: Vec::new(),
      next_packet_id: 1,
      latency_samples: Vec::new(),
    }
  }
}

fn runtime() -> &'static Mutex<Runtime> {
  static RUNTIME: OnceLock<Mutex<Runtime>> = OnceLock::new();
  RUNTIME.get_or_init(|| Mutex::new(Runtime::default()))
}

fn with_runtime<F, T>(f: F) -> Result<T, i32>
where
  F: FnOnce(&mut Runtime) -> Result<T, i32>,
{
  let mut guard = runtime().lock().map_err(|_| -1)?;
  f(&mut guard)
}

fn log(runtime: &mut Runtime, level: &str, message: impl Into<String>) {
  runtime.event_log.push(LogEntry {
    timestamp_ms: unix_timestamp_ms(),
    level: level.into(),
    message: message.into(),
  });
  if runtime.event_log.len() > 2_000 {
    let drain = runtime.event_log.len() - 2_000;
    runtime.event_log.drain(0..drain);
  }
}

fn push_packet(
  runtime: &mut Runtime,
  packet_type: &str,
  source: &str,
  destination: &str,
  ttl: u8,
  sequence: u32,
  size_bytes: usize,
  direction: &str,
) {
  let id = runtime.next_packet_id;
  runtime.next_packet_id += 1;
  runtime.packet_log.push(PacketRecord {
    id,
    packet_type: packet_type.into(),
    source: source.into(),
    destination: destination.into(),
    ttl,
    sequence,
    timestamp_ms: unix_timestamp_ms(),
    size_bytes,
    direction: direction.into(),
  });
  if runtime.packet_log.len() > 500 {
    runtime.packet_log.remove(0);
  }
}

fn record_event(runtime: &mut Runtime, event: &SdkEvent) {
  match event {
    SdkEvent::NetworkJoined => log(runtime, "info", "network joined"),
    SdkEvent::NetworkLeft => log(runtime, "info", "network left"),
    SdkEvent::PeerDiscovered { node_id, name } => {
      log(runtime, "info", format!("peer discovered: {name} ({node_id})"));
    }
    SdkEvent::PeerLost { node_id } => {
      log(runtime, "warn", format!("peer lost: {node_id}"));
    }
    SdkEvent::NeighborAdded(n) => {
      log(runtime, "info", format!("neighbor added: {}", n.node_name));
    }
    SdkEvent::NeighborRemoved { node_id } => {
      log(runtime, "warn", format!("neighbor removed: {node_id}"));
    }
    SdkEvent::LocationReceived { from, .. } => {
      runtime.stats.packets_received += 1;
      log(runtime, "debug", format!("location received from {from}"));
      push_packet(
        runtime,
        "location",
        &from.to_string(),
        "local",
        0,
        0,
        16,
        "inbound",
      );
    }
    SdkEvent::EmergencyReceived { from, emergency } => {
      runtime.stats.packets_received += 1;
      log(
        runtime,
        "error",
        format!("SOS from {from}: {}", emergency.message),
      );
    }
    SdkEvent::MessageReceived { from, message } => {
      runtime.stats.packets_received += 1;
      log(
        runtime,
        "debug",
        format!("message from {from}: {}", message.content),
      );
    }
    SdkEvent::VoiceFrameReceived { from } => {
      runtime.stats.voice_frames_received += 1;
      runtime.stats.packets_received += 1;
      log(runtime, "debug", format!("voice frame from {from}"));
      push_packet(
        runtime,
        "voice",
        &from.to_string(),
        "local",
        0,
        0,
        320,
        "inbound",
      );
    }
  }
}

fn event_label(event: &SdkEvent) -> String {
  match event {
    SdkEvent::NetworkJoined => "network_joined".into(),
    SdkEvent::NetworkLeft => "network_left".into(),
    SdkEvent::PeerDiscovered { name, .. } => format!("peer_discovered:{name}"),
    SdkEvent::PeerLost { .. } => "peer_lost".into(),
    SdkEvent::NeighborAdded(n) => format!("neighbor_added:{}", n.node_name),
    SdkEvent::NeighborRemoved { .. } => "neighbor_removed".into(),
    SdkEvent::LocationReceived { .. } => "location_received".into(),
    SdkEvent::EmergencyReceived { .. } => "emergency_received".into(),
    SdkEvent::MessageReceived { .. } => "message_received".into(),
    SdkEvent::VoiceFrameReceived { .. } => "voice_received".into(),
  }
}

fn json_string<T: serde::Serialize>(value: &T) -> Result<*mut c_char, i32> {
  let json = serde_json::to_string(value).map_err(|_| -2)?;
  CString::new(json).map(|s| s.into_raw()).map_err(|_| -2)
}

/// # Safety
/// `ptr` must be returned by this library and not freed twice.
#[no_mangle]
pub unsafe extern "C" fn conduit_free_string(ptr: *mut c_char) {
  if ptr.is_null() {
    return;
  }
  drop(unsafe { CString::from_raw(ptr) });
}

#[no_mangle]
pub extern "C" fn conduit_version() -> *mut c_char {
  json_string(&serde_json::json!({
    "crate": env!("CARGO_PKG_VERSION"),
    "protocol": "0.1",
  }))
  .unwrap_or(std::ptr::null_mut())
}

/// # Safety
/// `name` must be a valid UTF-8 null-terminated C string.
#[no_mangle]
pub unsafe extern "C" fn conduit_init(name: *const c_char) -> i32 {
  if name.is_null() {
    return -1;
  }
  let name = match CStr::from_ptr(name).to_str() {
    Ok(s) => s.to_string(),
    Err(_) => return -1,
  };

  with_runtime(|runtime| {
    let config = SdkConfig::builder().node_name(&name).build();
    let conduit = Conduit::initialize(config).map_err(|_| -2)?;
    runtime.conduit = Some(conduit);
    runtime.stats = NetworkStats::default();
    runtime.packet_log.clear();
    runtime.event_log.clear();
    runtime.latency_samples.clear();
    log(runtime, "info", format!("initialized as {name}"));
    Ok(0)
  })
  .unwrap_or(-1)
}

#[no_mangle]
pub extern "C" fn conduit_join() -> i32 {
  with_runtime(|runtime| {
    let conduit = runtime.conduit.as_mut().ok_or(-3)?;
    conduit.join_network().map_err(|_| -2)?;
    log(runtime, "info", "joined network");
    Ok(0)
  })
  .unwrap_or(-1)
}

#[no_mangle]
pub extern "C" fn conduit_leave() -> i32 {
  with_runtime(|runtime| {
    let conduit = runtime.conduit.as_mut().ok_or(-3)?;
    conduit.leave_network().map_err(|_| -2)?;
    log(runtime, "info", "left network");
    Ok(0)
  })
  .unwrap_or(-1)
}

#[no_mangle]
pub extern "C" fn conduit_tick() -> *mut c_char {
  let result = with_runtime(|runtime| {
    let started = unix_timestamp_ms();
    let events = {
      let conduit = runtime.conduit.as_mut().ok_or(-3)?;
      conduit.tick().map_err(|_| -2)?
    };
    let elapsed = unix_timestamp_ms().saturating_sub(started);
    runtime.latency_samples.push(elapsed);
    if runtime.latency_samples.len() > 100 {
      runtime.latency_samples.remove(0);
    }
    runtime.stats.current_rtt_ms = elapsed;
    runtime.stats.average_latency_ms = runtime
      .latency_samples
      .iter()
      .sum::<u64>()
      .checked_div(runtime.latency_samples.len() as u64)
      .unwrap_or(0);

    let labels: Vec<String> = events.iter().map(event_label).collect();
    for event in &events {
      record_event(runtime, event);
    }

    let diagnostics = runtime
      .conduit
      .as_ref()
      .ok_or(-3)?
      .diagnostics();
    runtime.stats.connected_nodes = diagnostics.neighbor_count;
    runtime.stats.routes = diagnostics.route_count;

    let playback = runtime
      .conduit
      .as_mut()
      .ok_or(-3)?
      .voice_mut()
      .playback()
      .ok()
      .flatten();

    let response = TickResponse {
      events: labels,
      stats: runtime.stats.clone(),
      diagnostics,
      playback_samples: playback,
    };
    json_string(&response)
  });

  match result {
    Ok(ptr) => ptr,
    Err(_) => std::ptr::null_mut(),
  }
}

#[no_mangle]
pub extern "C" fn conduit_set_ptt(active: u8) -> i32 {
  with_runtime(|runtime| {
    let conduit = runtime.conduit.as_mut().ok_or(-3)?;
    conduit.set_push_to_talk(active != 0);
    log(
      runtime,
      "info",
      if active != 0 {
        "push-to-talk pressed"
      } else {
        "push-to-talk released"
      },
    );
    Ok(0)
  })
  .unwrap_or(-1)
}

/// # Safety
/// `mode` must be `ptt`, `continuous`, or `vad`.
#[no_mangle]
pub unsafe extern "C" fn conduit_set_voice_mode(mode: *const c_char) -> i32 {
  if mode.is_null() {
    return -1;
  }
  let mode = match CStr::from_ptr(mode).to_str() {
    Ok(s) => s,
    Err(_) => return -1,
  };
  let voice_mode = match mode {
    "ptt" => VoiceMode::PushToTalk,
    "continuous" => VoiceMode::Continuous,
    "vad" => VoiceMode::VoiceActivity,
    _ => return -1,
  };
  with_runtime(|runtime| {
    let conduit = runtime.conduit.as_mut().ok_or(-3)?;
    conduit.set_voice_mode(voice_mode);
    log(runtime, "info", format!("voice mode set to {mode}"));
    Ok(0)
  })
  .unwrap_or(-1)
}

/// # Safety
/// `samples` must point to `len` valid i16 values.
#[no_mangle]
pub unsafe extern "C" fn conduit_send_voice(samples: *const i16, len: usize) -> i32 {
  if samples.is_null() || len == 0 {
    return -1;
  }
  let slice = unsafe { std::slice::from_raw_parts(samples, len) };
  with_runtime(|runtime| {
    let node_id = runtime
      .conduit
      .as_ref()
      .ok_or(-3)?
      .node_id()
      .to_string();
    let conduit = runtime.conduit.as_mut().ok_or(-3)?;
    conduit.send_voice(slice).map_err(|_| -2)?;
    runtime.stats.voice_frames_sent += 1;
    runtime.stats.packets_sent += 1;
    runtime.stats.bandwidth_bytes += len as u64 * 2;
    push_packet(
      runtime,
      "voice",
      &node_id,
      "broadcast",
      8,
      0,
      len * 2,
      "outbound",
    );
    Ok(0)
  })
  .unwrap_or(-1)
}

#[no_mangle]
pub extern "C" fn conduit_get_diagnostics() -> *mut c_char {
  let result = with_runtime(|runtime| {
    let conduit = runtime.conduit.as_ref().ok_or(-3)?;
    json_string(&conduit.diagnostics())
  });
  match result {
    Ok(ptr) => ptr,
    Err(_) => std::ptr::null_mut(),
  }
}

#[no_mangle]
pub extern "C" fn conduit_get_packet_log() -> *mut c_char {
  let result = with_runtime(|runtime| json_string(&runtime.packet_log));
  match result {
    Ok(ptr) => ptr,
    Err(_) => std::ptr::null_mut(),
  }
}

#[no_mangle]
pub extern "C" fn conduit_get_event_log() -> *mut c_char {
  let result = with_runtime(|runtime| json_string(&runtime.event_log));
  match result {
    Ok(ptr) => ptr,
    Err(_) => std::ptr::null_mut(),
  }
}

#[no_mangle]
pub extern "C" fn conduit_export_logs() -> *mut c_char {
  let result = with_runtime(|runtime| {
    json_string(&serde_json::json!({
      "exported_at_ms": unix_timestamp_ms(),
      "stats": runtime.stats,
      "packets": runtime.packet_log,
      "events": runtime.event_log,
    }))
  });
  match result {
    Ok(ptr) => ptr,
    Err(_) => std::ptr::null_mut(),
  }
}

#[cfg(test)]
mod tests {
  use super::*;
  use std::ffi::CString;

  #[test]
  fn init_join_tick() {
    unsafe {
      let name = CString::new("test-node").unwrap();
      assert_eq!(conduit_init(name.as_ptr()), 0);
      assert_eq!(conduit_join(), 0);
      let ptr = conduit_tick();
      assert!(!ptr.is_null());
      let json = CStr::from_ptr(ptr).to_str().unwrap();
      assert!(json.contains("test-node"));
      conduit_free_string(ptr);
      assert_eq!(conduit_leave(), 0);
    }
  }
}
