# Conduit

> **Build communication networks, not connections.**

Conduit is an open-source communication platform designed to create self-organizing, low-latency mesh networks between nearby devices without relying on the Internet or centralized infrastructure.

Unlike traditional voice chat applications that require servers, Conduit is designed around the idea that **every device is an equal participant**. Each device can discover nearby peers, exchange information, relay packets when necessary, and recover automatically when nodes join or leave.

Although the first implementation targets **motorcycle intercoms**, Conduit is intentionally designed as a **general-purpose communication platform** capable of supporting many applications, including:

* Motorcycle group communication
* Cycling groups
* Hiking expeditions
* Hotel staff communication
* Warehouse coordination
* Emergency response teams
* Industrial facilities
* Local offline messaging

The motorcycle intercom is **an application built on top of Conduit**, not the definition of Conduit itself.

---

# Philosophy

Traditional communication systems are built around servers.

```
Client
   │
Server
   │
Client
```

Conduit removes the server entirely.

```
Node ←→ Node ←→ Node
  ↑        ↑        ↑
Node ←→ Node ←→ Node
```

Every node is:

* Client
* Relay
* Router
* Packet processor

No device is more important than another.

The network should continue operating even if multiple devices disappear.

---

# Design Goals

## Functional Goals

* No Internet required
* Infrastructure-free communication
* Automatic nearby discovery
* Self-healing network
* Dynamic routing
* Low latency voice
* Extensible packet system
* Modular architecture
* Cross-platform application layer

## Engineering Goals

* Clean separation of responsibilities
* Replaceable networking technologies
* Easy to extend
* Easy to debug
* Easy to test
* Open protocol
* Deterministic packet processing

---

# Architecture

```
Applications
──────────────────────────────────────
Ride Intercom
Hotel Communication
Warehouse Communication
Emergency Response

                │

Conduit SDK
──────────────────────────────────────
Voice API
Messaging API
Location API
Emergency API

                │

Conduit Services
──────────────────────────────────────
Voice
Mesh
Routing
Discovery
Security

                │

Transport Layer
──────────────────────────────────────
Packet Manager
Serialization
Compression
Encryption

                │

Network Drivers
──────────────────────────────────────
Wi-Fi Aware
Wi-Fi Direct
Hotspot
Future Drivers

                │

Hardware
```

Every layer has exactly one responsibility.

---

# Repository Structure

```
conduit/

├── docs/
├── examples/

├── conduit-core/
├── conduit-sdk/

├── conduit-discovery/
├── conduit-mesh/
├── conduit-routing/

├── conduit-transport/
├── conduit-security/
├── conduit-voice/

├── conduit-cli/
├── conduit-tools/

└── applications/
```

---

# Development Flow

Conduit should **never** be developed from the top down.

Instead, every layer should be independently designed, implemented, tested, and stabilized before moving upward.

---

# Phase 1

## Core

Everything depends on Core.

Responsibilities

* Configuration
* Logging
* Utilities
* IDs
* Versioning
* Serialization helpers
* Packet definitions
* Constants
* Event system

Deliverables

```
Conduit Core

✓ Logging

✓ Packet Models

✓ Configuration

✓ Serialization

✓ Utilities
```

Nothing should communicate yet.

---

# Phase 2

## Packet System

Before networking exists, packet definitions must exist.

Every packet should share one header.

```
Header

Version
Packet Type
Priority
Flags
TTL
Sequence Number
Timestamp
Source Node
Destination
Payload Length
Checksum
```

Payload depends on packet type.

Possible packet types

Voice

Location

Heartbeat

Emergency

Messaging

Discovery

Routing

Telemetry

Control

Future packet types should require **no routing changes**.

---

# Phase 3

## Transport Layer

The transport layer knows nothing about motorcycles.

Responsibilities

* Packet serialization
* Packet parsing
* Fragmentation
* Reassembly
* Encryption
* Compression

Input

```
Packet Object
```

Output

```
Byte Stream
```

---

# Phase 4

## Discovery

Devices need to discover one another.

The discovery engine should support multiple implementations.

```
Discovery Interface

↓

Wi-Fi Aware

↓

Wi-Fi Direct

↓

Hotspot

↓

Future Technologies
```

The rest of the system must never know which discovery implementation is being used.

---

# Phase 5

## Mesh Engine

This is where nodes become neighbors.

Responsibilities

Maintain

* Neighbor table
* Node IDs
* Last seen timestamps
* Signal quality
* Link quality
* Heartbeats

The mesh engine never routes packets.

It only knows who is nearby.

---

# Phase 6

## Routing Engine

The routing engine is the heart of Conduit.

Responsibilities

* Packet forwarding
* Route selection
* Duplicate detection
* TTL handling
* Congestion handling
* Neighbor scoring
* Loop prevention

Example

```
A

↓

B

↓

C

↓

D
```

If B disappears

```
A

↓

E

↓

C

↓

D
```

The network automatically recovers.

No manual intervention.

No designated master.

---

# Phase 7

## Security

Every packet should be authenticated.

Responsibilities

* Encryption
* Authentication
* Session Keys
* Replay Protection
* Identity Verification

Security should exist below applications.

Applications should never perform encryption themselves.

---

# Phase 8

## Voice Engine

Voice becomes another packet type.

Pipeline

```
Microphone

↓

Noise Suppression

↓

Echo Cancellation

↓

Voice Activity Detection

↓

Opus Encoder

↓

Packetization

↓

Routing

↓

Network

↓

Receive

↓

Jitter Buffer

↓

Opus Decoder

↓

Playback
```

The routing engine never knows the packet contains voice.

---

# Phase 9

## SDK

Applications communicate through the SDK.

Example

```
Conduit.initialize()

Conduit.joinNetwork()

Conduit.sendVoice()

Conduit.sendLocation()

Conduit.sendEmergency()
```

Applications never interact directly with routing or networking.

---

# Phase 10

## Applications

Applications are simply clients.

Possible applications

Ride Intercom

Warehouse Communication

Hotel Staff

Cycling

Search and Rescue

Emergency Teams

Industrial Communication

---

# Packet Flow

Imagine Rider A speaks.

```
Microphone

↓

Voice Engine

↓

Packet Manager

↓

Routing

↓

Transport

↓

Network

↓

Nearby Node

↓

Routing

↓

Voice Engine

↓

Speaker
```

Every module performs only one job.

---

# Guiding Principles

## Single Responsibility

Every module should have exactly one purpose.

---

## Replaceability

Every module should be replaceable without changing higher layers.

Example

Replace

```
Wi-Fi Direct
```

with

```
Wi-Fi Aware
```

Nothing above the driver changes.

---

## Extensibility

Adding GPS should never require changing routing.

Adding video should never require changing discovery.

Adding telemetry should never require changing transport.

Everything grows horizontally.

---

## Infrastructure Independence

Conduit should never require

* Internet
* Cloud services
* Dedicated servers
* Vendor APIs

The network belongs entirely to the participating devices.

---

## Open Protocol

The protocol specification should be fully documented.

Third-party developers should be able to build compatible implementations.

---

# Roadmap

## Milestone 1

Core

Packet System

Transport

Documentation

---

## Milestone 2

Discovery

Local Communication

Neighbor Management

---

## Milestone 3

Routing

Packet Relay

Self-Healing Network

---

## Milestone 4

Voice Engine

Push-To-Talk

Voice Activation

---

## Milestone 5

Android Application

Capacitor Integration

Bluetooth Headset Support

---

## Milestone 6

Ride Intercom

Group Management

Emergency Features

GPS Sharing

---

## Milestone 7

Public SDK

Developer Documentation

Example Applications

---

# Long-Term Vision

Conduit is not intended to be another voice chat application.

It is intended to become a **distributed communication operating layer** for nearby devices.

Applications should think in terms of communication primitives rather than networking.

```
Application

↓

Conduit SDK

↓

Conduit Platform

↓

Any Supported Network Technology
```

If a better networking technology appears in the future, only the lowest layer changes.

Everything above it continues to function without modification.

That is the ultimate objective of Conduit.
