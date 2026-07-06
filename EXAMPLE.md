# Conduit Android Intercom

> A reference Android application for testing the Conduit framework under real-world conditions.

---

## Purpose

The Android Intercom is **not intended to be a production application**.

Its primary purpose is to validate and improve the Conduit framework by providing a practical environment for testing:

* Device discovery
* Mesh formation
* Packet routing
* Voice transport
* Recovery from node failures
* Latency
* Reliability
* Battery consumption
* Bluetooth headset compatibility
* Network performance while moving

This application serves as the first real-world implementation of Conduit.

---

# Goals

The application should remain intentionally simple.

Its objective is **testing the framework**, not building end-user features.

Primary goals include:

* Discover nearby devices
* Form communication groups
* Join and leave dynamically
* Capture microphone audio
* Send and receive voice packets
* Display routing information
* Visualize network topology
* Measure latency
* Log packet statistics
* Simulate real riding conditions

---

# Technology Stack

Frontend

* Capacitor
* HTML
* CSS
* TypeScript

Native Layer

* Android (Kotlin)
* Capacitor Plugins

Framework

* Rust
* Conduit Crates

Communication

* UDP
* Opus Codec
* Native Android Networking APIs

---

# Development Philosophy

The application should contain **no communication logic**.

Instead:

```
Application

↓

Conduit SDK

↓

Conduit Framework

↓

Android Networking
```

The application only provides a user interface.

Every networking decision belongs inside Conduit.

---

# Planned Features

## Device Discovery

Display nearby Conduit nodes.

Information includes:

* Node ID
* Device Name
* Signal Strength
* Connection Quality
* Last Seen

---

## Mesh Visualization

Display the current mesh topology.

Example:

```
Phone A

│

Phone B

├──── Phone C

└──── Phone D
```

This view exists purely for debugging.

---

## Voice Testing

Simple controls:

* Push-To-Talk
* Continuous Voice
* Voice Activity Detection

Display:

* Current codec
* Packet rate
* Latency
* Packet loss

---

## Network Statistics

Display:

* Connected Nodes
* Routes
* Forwarded Packets
* Duplicate Packets
* Packet Drops
* Current RTT
* Average Latency
* Bandwidth Usage

---

## Packet Inspector

Developer screen showing:

* Packet ID
* Packet Type
* TTL
* Route
* Timestamp
* Size

Useful when debugging routing behavior.

---

## Logging

Every important event should be recorded.

Examples:

* Node joined
* Node disconnected
* Route updated
* Packet forwarded
* Packet dropped
* Voice stream started
* Voice stream stopped

Logs should be exportable for analysis.

---

# Test Modes

## Single Device

Verify:

* Audio capture
* Audio playback
* Packet generation

---

## Two Devices

Verify:

* Discovery
* Direct communication
* Voice quality

---

## Three Devices

Verify:

* Routing
* Packet forwarding
* Neighbor updates

---

## Mesh Test

Use multiple Android devices.

Observe:

* Dynamic routing
* Route recovery
* Duplicate suppression
* Forwarding efficiency

---

## Mobility Test

Move devices apart.

Observe:

* Route changes
* Recovery time
* Voice interruptions
* Reconnection speed

---

## Battery Test

Run continuously.

Measure:

* CPU usage
* Memory usage
* Battery consumption
* Thermal behavior

---

# Future Testing

Additional testing may include:

* Motorcycle rides
* Cycling groups
* Hotel communication
* Warehouse environments
* Emergency response simulations

---

# Development Roadmap

## Phase 1

* Android project setup
* Capacitor integration
* Rust library integration

---

## Phase 2

* Discovery testing
* Device list
* Connection diagnostics

---

## Phase 3

* Voice transmission
* Packet inspection
* Network statistics

---

## Phase 4

* Mesh routing
* Relay testing
* Multi-device communication

---

## Phase 5

* Performance optimization
* Battery optimization
* Bluetooth headset testing

---

## Success Criteria

The reference application is considered successful when it can:

* Discover nearby Conduit nodes automatically.
* Form a resilient communication network.
* Recover from node failures without user intervention.
* Deliver intelligible voice with minimal latency.
* Operate for extended periods on standard Android devices.
* Provide sufficient diagnostics to improve the Conduit framework.

---

# Project Scope

This application exists solely as a testing and validation tool.

It intentionally avoids product-specific functionality such as user accounts, ride planning, navigation, messaging interfaces, or polished user experiences.

Those belong in future applications.

The only responsibility of this project is to verify that the Conduit framework performs reliably in real-world environments.
