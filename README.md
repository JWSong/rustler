# Rustler

> ⚠️ Work in Progress: This project is under active development and the API may change.

A lightweight, async-first Gossip-based Failure Detection library written in Rust. Rustler helps you build reliable distributed systems by providing robust node failure detection through gossip protocols.

## Features

- Flexible Transport Layer: Built-in support for both UDP and TCP protocols
- Configurable Failure Detection: Customizable gossip intervals, suspect timeouts, and fanout parameters
- State Machine Based: Robust node state tracking using a finite state machine (Alive → Suspect → Dead)
- Async First: Built on Tokio for efficient async I/O
- Extensible: Implement the NetworkTransport trait for custom transport protocols

## Quick Start

Basic usage with UDP:

```rust
use rustler::failure_detector::FailureDetector;
use rustler::transport::UdpTransport;

let transport = UdpTransport::new(SocketAddr::from(([127, 0, 0, 1], 8080))).await.unwrap();
let failure_detector = FailureDetector::builder(SocketAddr::from(([127, 0, 0, 1], 8080))).build();
failure_detector.run(transport).await;
```
