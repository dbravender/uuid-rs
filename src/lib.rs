//! World's fastest uuid microservice* that I'm aware of.
//!
//! This crate exposes a deliberately tiny gRPC service that returns real `UUIDv4`
//! values as raw 16-byte Protobuf payloads. Clients serialize those bytes into
//! canonical UUID strings on the receiving side.

/// Generated Protobuf and gRPC bindings.
#[allow(
    missing_docs,
    unused_qualifications,
    clippy::all,
    clippy::cargo,
    clippy::nursery,
    clippy::pedantic
)]
pub mod proto {
    tonic::include_proto!("uuid.v1");
}

/// UUID generation service helpers.
pub mod uuid_service;
