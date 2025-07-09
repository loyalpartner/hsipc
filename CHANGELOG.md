# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Initial release of hsipc
- RPC system v2 with jsonrpsee-style macros
- Support for async/sync methods
- Subscription system with PendingSubscriptionSink
- Comprehensive test suite with TDD workflow
- Performance benchmarks
- Smart test selection based on file changes
- Complete documentation and examples

### Changed
- N/A

### Deprecated
- N/A

### Removed
- N/A

### Fixed
- N/A

### Security
- N/A

## [0.1.0] - 2025-07-09

### Added
- Initial release of hsipc
- High-performance inter-process communication framework
- RPC system with `#[rpc(server, client, namespace = "name")]` macro
- Support for both synchronous and asynchronous methods
- Subscription system with real-time event streaming
- Type-safe service definitions and client generation
- Built on ipmb for maximum performance
- Comprehensive examples and documentation
- Complete test suite with 13 passing tests
- Performance benchmarks showing 596-739 MiB/s throughput
- Smart testing workflow with TDD support

### Technical Details
- **Core Features**: ProcessHub, Service trait, Event system
- **Performance**: 596-739 MiB/s message throughput, ~21.4µs event latency
- **Testing**: 13 tests passing, 30-second validation cycle
- **Examples**: trait_based_service, request_response, pubsub_events
- **Documentation**: Complete API docs, architecture guides, performance specs