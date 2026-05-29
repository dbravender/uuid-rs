# Rust UUID gRPC Service Design

## Summary

Build a publishable Rust project for a single-purpose UUID microservice with the tagline:

> The world's fastest uuid microservice* that I'm aware of

The service is intentionally funny in concept and serious in execution. It exposes one gRPC method backed by Protobuf, returns real UUIDv4 values, keeps the wire format lean, and documents the joke inspiration at <https://x.com/paulbohm/status/2052898355219517708?s=20>.

## Goals

- Provide one gRPC endpoint that returns one real UUIDv4 per request.
- Use Protobuf as the single wire protocol through `tonic` and `prost`.
- Return UUIDs as raw 16-byte Protobuf `bytes`, requiring clients to serialize or format them into canonical UUID strings on their side.
- Ship as a polished Rust project: library core, server binary, generated protobuf integration, docs, benchmarks, examples, strict linting, formatting, and pre-commit checks through `prek`.
- Use the latest stable Rust toolchain via the `stable` channel instead of relying on nightly features.
- Provide honest benchmark artifacts and repeatable commands for both internal hot-path benchmarks and real gRPC load testing.

## Non-Goals

- Do not expose REST, JSON, GraphQL, WebSocket, or multiple protocol variants.
- Do not return UUID strings from the service hot path.
- Do not implement UUID generation by hand when the `uuid` crate already provides correct, maintained UUIDv4 generation.
- Do not optimize by compromising UUID correctness, gRPC compatibility, or benchmark reproducibility.
- Do not require a database, external entropy service, or distributed coordination layer.

## Architecture

The crate will have a small library core and a server binary.

- `proto/uuid_service.proto` defines one package, one service, one request message, and one response message.
- `build.rs` compiles the Protobuf definition with `tonic-build`.
- `src/lib.rs` exposes generated proto modules and the service implementation.
- `src/uuid_service.rs` contains the gRPC service implementation.
- `src/main.rs` starts the gRPC server and accepts basic CLI configuration such as listen address.
- `benches/` contains Criterion benchmarks for UUID generation and Protobuf response construction.
- `examples/` contains a small client that calls the service and formats the returned bytes into a canonical UUID string, making the client-side serialization requirement visible.

The service method is:

```protobuf
service Uuid {
  rpc Generate(GenerateRequest) returns (GenerateResponse);
}

message GenerateRequest {}

message GenerateResponse {
  bytes uuid = 1;
}
```

The response field must contain exactly 16 bytes. The server generates a real UUIDv4 using the `uuid` crate, copies the UUID bytes into the Protobuf response, and does no string formatting in the request path.

## Data Flow

1. A client sends an empty `GenerateRequest` over gRPC.
2. The server generates a UUIDv4 with `Uuid::new_v4()`.
3. The server returns `GenerateResponse { uuid: <16 raw bytes> }`.
4. The client validates the length and converts the bytes into a UUID value.
5. The client formats the UUID as a canonical hyphenated string only after receiving the response.

This keeps the service's wire protocol optimized while preserving fully valid UUID semantics.

## Error Handling

The server path should have no normal request-level failure modes beyond transport failures. UUID generation through the `uuid` crate is expected to be infallible for normal operation.

Client examples must treat any response with a byte length other than 16 as invalid and return a clear error. That error path documents the protocol contract without adding extra server features.

## Benchmarks

The project will include two benchmark layers:

- Criterion benchmarks for the local hot path:
  - UUIDv4 generation.
  - Converting UUID bytes into `GenerateResponse`.
  - Optional canonical string formatting on the client side to show the cost the service avoids.
- A documented real gRPC benchmark using a standard load generator such as `ghz`:
  - Start the release-mode server.
  - Run a fixed-duration unary RPC load test.
  - Record command, machine notes, concurrency, request rate, latency percentiles, and throughput.

Benchmark claims in the README must be framed as reproducible local results, not universal superiority. The asterisk in the tagline is part of the product voice and should point to the benchmark methodology.

## Tooling And Quality

The project will include:

- `Cargo.toml` metadata ready for publication, with license, categories, keywords, README, crate description, and a repository URL only when the final public repository URL is known.
- `rust-toolchain.toml` set to the stable channel with `rustfmt` and `clippy` components.
- Strict Rust lints in `Cargo.toml`, including denial of common correctness hazards and pedantic Clippy warnings with targeted allows where the project deliberately stays pragmatic.
- `rustfmt.toml` for stable formatting configuration.
- `.pre-commit-config.yaml` configured for `prek`, running formatting, Clippy, tests, and documentation checks.
- README docs covering installation, running the server, calling it from a client, wire protocol details, benchmarks, and the inspiration link.
- Crate-level docs that explain why the service returns bytes and how clients should serialize them.

## Testing

Tests will be written before implementation for each behavior:

- Unit tests confirm generated responses contain exactly 16 bytes.
- Unit tests confirm the UUID version and variant bits represent real UUIDv4 values.
- Integration tests call the gRPC service in-process and validate the response contract.
- Client-side tests verify successful byte-to-UUID formatting and rejection of invalid byte lengths.
- Tooling verification runs `cargo fmt --check`, strict `cargo clippy`, `cargo test`, `cargo doc --no-deps`, and `cargo bench --no-run`; the external `ghz` benchmark is documented as a manual release-performance command.

## Release Readiness

The finished project should be ready for a normal Rust publication flow:

- `cargo package` succeeds.
- README and crate docs are useful without private context.
- The gRPC API is minimal and stable enough for a first release.
- Benchmarks can be re-run by another developer.
- Pre-commit checks work through `prek`.
