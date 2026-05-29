# Rust UUID gRPC Service Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build a publishable Rust gRPC service with one Protobuf endpoint that returns real UUIDv4 values as raw 16-byte payloads.

**Architecture:** The crate has a library core, a `uuid-rs` server binary, generated Protobuf modules, an example client that serializes UUID bytes on the receiving side, Criterion benchmarks, and publication-quality docs/tooling. The hot path generates `Uuid::new_v4()`, returns `GenerateResponse { uuid: Vec<u8> }`, and deliberately does no UUID string formatting in the service.

**Tech Stack:** Rust stable 1.96.0 channel, exact direct dependency pins published on or before 2026-05-22, `tonic` 0.14.6, `tonic-prost` 0.14.6, `tonic-prost-build` 0.14.6, `prost` 0.14.3, `tokio` 1.52.3, `uuid` 1.23.1, `clap` 4.6.1, `criterion` 0.8.2 without default Plotters/Rayon features, `protoc-bin-vendored` 3.2.0, `cargo-quarantine`, `prek`.

---

## File Structure

- Create `Cargo.toml`: package metadata, dependencies, and lint policy.
- Create `Cargo.lock`: committed application lockfile after dependency-age verification.
- Create `rust-toolchain.toml`: stable toolchain plus `rustfmt` and `clippy`.
- Create `rustfmt.toml`: stable formatting policy.
- Create `.gitignore`: target/build/editor ignores.
- Create `cooldown.toml`: off-the-shelf lockfile downgrade policy.
- Create `quarantine.toml`: off-the-shelf minimum dependency age verification policy.
- Create `.pre-commit-config.yaml`: `prek` hooks for format, clippy, tests, docs, package dry run.
- Create `proto/uuid_service.proto`: single gRPC service and raw bytes response.
- Create `build.rs`: Protobuf compilation using vendored `protoc`.
- Create `src/lib.rs`: public crate docs, generated proto export, service exports.
- Create `src/uuid_service.rs`: service implementation and byte-to-UUID helper.
- Create `src/main.rs`: server binary with address CLI.
- Create `examples/client.rs`: gRPC client that converts bytes to a canonical UUID string.
- Create `tests/grpc_contract.rs`: end-to-end in-process gRPC contract tests.
- Create `benches/uuid_hot_path.rs`: Criterion hot-path benchmarks.
- Create `README.md`: usage, protocol, benchmark method, inspiration link.

---

### Task 1: Toolchain, Manifest, And Protocol Skeleton

**Files:**
- Create: `Cargo.toml`
- Create: `rust-toolchain.toml`
- Create: `rustfmt.toml`
- Create: `.gitignore`
- Create: `proto/uuid_service.proto`
- Create: `build.rs`

- [ ] **Step 1: Write the failing package/build test**

Run:

```bash
cargo metadata --format-version 1 --no-deps
```

Expected: FAIL because `Cargo.toml` does not exist.

- [ ] **Step 2: Create project manifest and toolchain files**

Create `Cargo.toml`:

```toml
[package]
name = "uuid-rs"
version = "0.1.0"
edition = "2024"
rust-version = "1.96"
license = "MIT OR Apache-2.0"
description = "The world's fastest uuid microservice* that I'm aware of."
readme = "README.md"
categories = ["network-programming", "web-programming"]
keywords = ["uuid", "grpc", "protobuf", "microservice"]
include = [
  "Cargo.toml",
  "README.md",
  "Cargo.lock",
  "build.rs",
  "cooldown.toml",
  "proto/**/*.proto",
  "src/**/*.rs",
  "examples/**/*.rs",
  "tests/**/*.rs",
  "benches/**/*.rs",
  "quarantine.toml",
  "rust-toolchain.toml",
  "rustfmt.toml",
]

[lib]
name = "uuid_rs"
path = "src/lib.rs"

[dependencies]
anyhow = "=1.0.102"
clap = { version = "=4.6.1", features = ["derive", "env"] }
prost = "=0.14.3"
tokio = { version = "=1.52.3", features = ["macros", "rt-multi-thread", "signal"] }
tonic = { version = "=0.14.6", features = ["transport"] }
tonic-prost = "=0.14.6"
uuid = { version = "=1.23.1", features = ["v4"] }

[build-dependencies]
protoc-bin-vendored = "=3.2.0"
tonic-prost-build = "=0.14.6"

[dev-dependencies]
criterion = { version = "=0.8.2", default-features = false, features = ["cargo_bench_support"] }

[lints.rust]
missing_docs = "deny"
unsafe_code = "deny"
unused_qualifications = "warn"

[lints.clippy]
all = "deny"
pedantic = "deny"
nursery = "deny"
cargo = "warn"
multiple_crate_versions = "allow"
cargo_common_metadata = "allow"
module_name_repetitions = "allow"
missing_errors_doc = "allow"
missing_panics_doc = "allow"
```

Create `rust-toolchain.toml`:

```toml
[toolchain]
channel = "stable"
components = ["clippy", "rustfmt"]
```

Create `rustfmt.toml`:

```toml
edition = "2024"
max_width = 100
newline_style = "Unix"
use_field_init_shorthand = true
use_try_shorthand = true
```

Create `.gitignore`:

```gitignore
/target/
/.idea/
/.vscode/
.DS_Store
*.profraw
```

- [ ] **Step 3: Create the Protobuf service definition**

Create `proto/uuid_service.proto`:

```protobuf
syntax = "proto3";

package uuid.v1;

service Uuid {
  rpc Generate(GenerateRequest) returns (GenerateResponse);
}

message GenerateRequest {}

message GenerateResponse {
  bytes uuid = 1;
}
```

- [ ] **Step 4: Create the build script**

Create `build.rs`:

```rust
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    let protoc = protoc_bin_vendored::protoc_bin_path()?;
    let mut prost_config = tonic_prost_build::Config::new();
    prost_config.protoc_executable(protoc);

    tonic_prost_build::configure().compile_with_config(
        prost_config,
        &["proto/uuid_service.proto"],
        &["proto"],
    )?;
    println!("cargo:rerun-if-changed=proto/uuid_service.proto");
    Ok(())
}
```

- [ ] **Step 5: Run metadata/build to verify the skeleton**

Run:

```bash
cargo metadata --format-version 1 --no-deps
cargo build
```

Expected: metadata succeeds; build fails because `src/lib.rs` and `src/main.rs` do not exist yet.

- [ ] **Step 6: Commit**

```bash
git add Cargo.toml rust-toolchain.toml rustfmt.toml .gitignore proto/uuid_service.proto build.rs
git commit -m "chore: scaffold uuid grpc crate"
```

---

### Task 2: Library Core And UUID Contract Tests

**Files:**
- Create: `src/lib.rs`
- Create: `src/uuid_service.rs`

- [ ] **Step 1: Write the failing unit tests**

Create `src/uuid_service.rs` with tests first:

```rust
use uuid::Uuid as RawUuid;

use crate::proto::GenerateResponse;

/// Generate one real UUIDv4 response as raw 16-byte Protobuf bytes.
#[must_use]
pub fn generate_response() -> GenerateResponse {
    unimplemented!("generate_response is implemented after the tests fail");
}

/// Convert a raw-byte UUID response into a UUID value for client-side formatting.
pub fn uuid_from_response(response: &GenerateResponse) -> Result<RawUuid, InvalidUuidBytes> {
    let bytes: [u8; 16] = response
        .uuid
        .as_slice()
        .try_into()
        .map_err(|_| InvalidUuidBytes { actual_len: response.uuid.len() })?;
    Ok(RawUuid::from_bytes(bytes))
}

/// Error returned when a response does not contain exactly 16 UUID bytes.
#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub struct InvalidUuidBytes {
    actual_len: usize,
}

impl InvalidUuidBytes {
    /// Return the byte length that was received.
    #[must_use]
    pub const fn actual_len(self) -> usize {
        self.actual_len
    }
}

impl std::fmt::Display for InvalidUuidBytes {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(formatter, "expected 16 UUID bytes, received {}", self.actual_len)
    }
}

impl std::error::Error for InvalidUuidBytes {}

#[cfg(test)]
mod tests {
    use uuid::{Variant, Version};

    use super::{generate_response, uuid_from_response, GenerateResponse};

    #[test]
    fn generate_response_returns_exactly_sixteen_uuid_bytes() {
        let response = generate_response();

        assert_eq!(response.uuid.len(), 16);
    }

    #[test]
    fn generate_response_returns_real_uuid_v4_bytes() {
        let response = generate_response();
        let uuid = uuid_from_response(&response).expect("response should be a UUID");

        assert_eq!(uuid.get_version(), Some(Version::Random));
        assert_eq!(uuid.get_variant(), Variant::RFC4122);
    }

    #[test]
    fn uuid_from_response_rejects_invalid_lengths() {
        let response = GenerateResponse { uuid: vec![1, 2, 3] };

        let error = uuid_from_response(&response).expect_err("three bytes is not a UUID");

        assert_eq!(error.actual_len(), 3);
        assert_eq!(error.to_string(), "expected 16 UUID bytes, received 3");
    }
}
```

Create minimal `src/lib.rs` so the tests compile far enough to fail on the `unimplemented!`:

```rust
//! World's fastest uuid microservice* that I'm aware of.
//!
//! This crate exposes a deliberately tiny gRPC service that returns real UUIDv4
//! values as raw 16-byte Protobuf payloads. Clients serialize those bytes into
//! canonical UUID strings on the receiving side.

/// Generated Protobuf and gRPC bindings.
#[allow(missing_docs)]
pub mod proto {
    tonic::include_proto!("uuid.v1");
}

/// UUID generation service helpers.
pub mod uuid_service;
```

- [ ] **Step 2: Run tests and verify the expected failure**

Run:

```bash
cargo test uuid_service
```

Expected: FAIL with `not implemented: generate_response is implemented after the tests fail`.

- [ ] **Step 3: Implement the minimal UUID response**

Replace `generate_response` in `src/uuid_service.rs`:

```rust
#[must_use]
pub fn generate_response() -> GenerateResponse {
    GenerateResponse {
        uuid: RawUuid::new_v4().into_bytes().to_vec(),
    }
}
```

- [ ] **Step 4: Run tests and verify green**

Run:

```bash
cargo test uuid_service
```

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add src/lib.rs src/uuid_service.rs
git commit -m "feat: add uuid response core"
```

---

### Task 3: gRPC Service, Server Binary, And Contract Test

**Files:**
- Modify: `src/uuid_service.rs`
- Create: `src/main.rs`
- Create: `tests/grpc_contract.rs`

- [ ] **Step 1: Write the failing gRPC contract test**

Create `tests/grpc_contract.rs`:

```rust
use std::net::SocketAddr;

use tokio::net::TcpListener;
use tokio::sync::oneshot;
use tokio::task::JoinHandle;
use tonic::transport::{Channel, Server};
use uuid::{Variant, Version};
use uuid_rs::proto::uuid_client::UuidClient;
use uuid_rs::proto::uuid_server::UuidServer;
use uuid_rs::proto::GenerateRequest;
use uuid_rs::uuid_service::{uuid_from_response, UuidGenerator};

async fn spawn_server() -> (SocketAddr, oneshot::Sender<()>, JoinHandle<Result<(), tonic::transport::Error>>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("bind test listener");
    let address = listener.local_addr().expect("read local addr");
    let stream = tokio_stream::wrappers::TcpListenerStream::new(listener);
    let (shutdown_tx, shutdown_rx) = oneshot::channel();

    let handle = tokio::spawn(async move {
        Server::builder()
            .add_service(UuidServer::new(UuidGenerator))
            .serve_with_incoming_shutdown(stream, async {
                let _ = shutdown_rx.await;
            })
            .await
    });

    (address, shutdown_tx, handle)
}

#[tokio::test]
async fn grpc_generate_returns_raw_uuid_v4_bytes() {
    let (address, shutdown_tx, handle) = spawn_server().await;
    let channel = Channel::from_shared(format!("http://{address}"))
        .expect("valid endpoint")
        .connect()
        .await
        .expect("connect to test server");
    let mut client = UuidClient::new(channel);

    let response = client
        .generate(GenerateRequest {})
        .await
        .expect("generate rpc succeeds")
        .into_inner();
    let uuid = uuid_from_response(&response).expect("valid UUID bytes");

    assert_eq!(response.uuid.len(), 16);
    assert_eq!(uuid.get_version(), Some(Version::Random));
    assert_eq!(uuid.get_variant(), Variant::RFC4122);

    shutdown_tx.send(()).expect("signal shutdown");
    handle.await.expect("server task joins").expect("server exits cleanly");
}
```

- [ ] **Step 2: Add missing test dependency and verify expected failure**

Add to `[dev-dependencies]` in `Cargo.toml`:

```toml
tokio-stream = { version = "=0.1.18", features = ["net"] }
```

Run:

```bash
cargo test --test grpc_contract
```

Expected: FAIL because `UuidGenerator` does not exist or does not implement the generated gRPC trait.

- [ ] **Step 3: Implement the gRPC service**

Append to `src/uuid_service.rs`:

```rust
use tonic::{Request, Response, Status};

use crate::proto::uuid_server::Uuid;
use crate::proto::GenerateRequest;

/// gRPC implementation for the UUID service.
#[derive(Debug, Default, Clone, Copy)]
pub struct UuidGenerator;

#[tonic::async_trait]
impl Uuid for UuidGenerator {
    async fn generate(
        &self,
        _request: Request<GenerateRequest>,
    ) -> Result<Response<GenerateResponse>, Status> {
        Ok(Response::new(generate_response()))
    }
}
```

- [ ] **Step 4: Create the server binary**

Append to `Cargo.toml`:

```toml
[[bin]]
name = "uuid-rs"
path = "src/main.rs"
```

Create `src/main.rs`:

```rust
use std::net::SocketAddr;

use clap::Parser;
use tonic::transport::Server;
use uuid_rs::proto::uuid_server::UuidServer;
use uuid_rs::uuid_service::UuidGenerator;

#[derive(Debug, Parser)]
#[command(version, about)]
struct Args {
    /// Address the gRPC server listens on.
    #[arg(long, env = "UUID_RS_LISTEN_ADDR", default_value = "127.0.0.1:50051")]
    listen: SocketAddr,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    Server::builder()
        .add_service(UuidServer::new(UuidGenerator))
        .serve(args.listen)
        .await?;

    Ok(())
}
```

- [ ] **Step 5: Run tests and verify green**

Run:

```bash
cargo test --test grpc_contract
cargo test
```

Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add Cargo.toml src/uuid_service.rs src/main.rs tests/grpc_contract.rs
git commit -m "feat: expose uuid grpc service"
```

---

### Task 4: Client Example And Client-Side Serialization Tests

**Files:**
- Create: `examples/client.rs`
- Modify: `src/uuid_service.rs`

- [ ] **Step 1: Write the failing example compilation check**

Run:

```bash
cargo build --example client
```

Expected: FAIL because `examples/client.rs` does not exist.

- [ ] **Step 2: Add the client-side serialization regression test**

Append this test to `src/uuid_service.rs` inside the existing `tests` module:

```rust
#[test]
fn client_side_formatting_produces_canonical_uuid_string() {
    let response = GenerateResponse {
        uuid: uuid::uuid!("67e55044-10b1-426f-9247-bb680e5fe0c8")
            .into_bytes()
            .to_vec(),
    };

    let uuid = uuid_from_response(&response).expect("valid UUID bytes");

    assert_eq!(uuid.hyphenated().to_string(), "67e55044-10b1-426f-9247-bb680e5fe0c8");
}
```

Run:

```bash
cargo test client_side_formatting_produces_canonical_uuid_string
```

Expected: PASS because `uuid_from_response` already owns this behavior; if it fails, fix only `uuid_from_response`.

- [ ] **Step 3: Create the example client**

Create `examples/client.rs`:

```rust
use clap::Parser;
use uuid_rs::proto::uuid_client::UuidClient;
use uuid_rs::proto::GenerateRequest;
use uuid_rs::uuid_service::uuid_from_response;

#[derive(Debug, Parser)]
#[command(version, about = "Call uuid-rs and serialize raw UUID bytes on the client side.")]
struct Args {
    /// gRPC endpoint to call.
    #[arg(long, env = "UUID_RS_ENDPOINT", default_value = "http://127.0.0.1:50051")]
    endpoint: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::parse();
    let mut client = UuidClient::connect(args.endpoint).await?;
    let response = client.generate(GenerateRequest {}).await?.into_inner();
    let uuid = uuid_from_response(&response)?;

    println!("{}", uuid.hyphenated());
    Ok(())
}
```

- [ ] **Step 4: Run example compilation and tests**

Run:

```bash
cargo test client_side_formatting_produces_canonical_uuid_string
cargo build --examples
```

Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add src/uuid_service.rs examples/client.rs
git commit -m "feat: add uuid client serialization example"
```

---

### Task 5: Benchmarks, Docs, Dependency Age, And Pre-Commit Checks

**Files:**
- Modify: `Cargo.toml`
- Create: `benches/uuid_hot_path.rs`
- Create: `README.md`
- Create: `cooldown.toml`
- Create: `quarantine.toml`
- Create: `.pre-commit-config.yaml`

- [ ] **Step 1: Declare and write benchmarks**

Append to `Cargo.toml`:

```toml
[[bench]]
name = "uuid_hot_path"
harness = false
```

Create `benches/uuid_hot_path.rs`:

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use uuid::Uuid as RawUuid;
use uuid_rs::uuid_service::{generate_response, uuid_from_response};

fn bench_uuid_generation(criterion: &mut Criterion) {
    criterion.bench_function("uuid_v4_generation", |bencher| {
        bencher.iter(|| black_box(RawUuid::new_v4()));
    });
}

fn bench_response_construction(criterion: &mut Criterion) {
    criterion.bench_function("generate_response_raw_bytes", |bencher| {
        bencher.iter(|| black_box(generate_response()));
    });
}

fn bench_client_side_formatting(criterion: &mut Criterion) {
    let response = generate_response();

    criterion.bench_function("client_side_hyphenated_formatting", |bencher| {
        bencher.iter(|| {
            let uuid = uuid_from_response(black_box(&response)).expect("benchmark UUID is valid");
            black_box(uuid.hyphenated().to_string());
        });
    });
}

criterion_group!(
    benches,
    bench_uuid_generation,
    bench_response_construction,
    bench_client_side_formatting
);
criterion_main!(benches);
```

- [ ] **Step 2: Write README**

Create `README.md`:

```markdown
# uuid-rs

> The world's fastest uuid microservice* that I'm aware of

`uuid-rs` is a real Rust gRPC service for generating real UUIDv4 values. It has one
endpoint, one Protobuf wire protocol, and one tiny trick: the service returns raw
16-byte UUID payloads, so the client serializes them into canonical UUID strings on
the other end.

Inspired by <https://x.com/paulbohm/status/2052898355219517708?s=20>.

It doesn't implement the database as described in the tweet since *that* would be stupid.

## Protocol

```protobuf
service Uuid {
  rpc Generate(GenerateRequest) returns (GenerateResponse);
}

message GenerateRequest {}

message GenerateResponse {
  bytes uuid = 1;
}
```

`GenerateResponse.uuid` is exactly 16 bytes. It is a valid UUIDv4 value. It is not a
string. This is performance engineering, apparently.

## Run

```bash
cargo run --release --bin uuid-rs -- --listen 127.0.0.1:50051
```

## Call

```bash
cargo run --example client -- --endpoint http://127.0.0.1:50051
```

The example client receives bytes, validates the length, constructs a UUID value,
and formats the canonical hyphenated string locally.

## Quality Checks

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps
cargo bench --no-run
cargo quarantine
cargo package
```

## Dependency Age Policy

Direct dependencies are exact-pinned to versions published at least seven days
before adoption. `Cargo.lock` is committed for the service binary, and
`cargo-quarantine` verifies every crates.io package in the lockfile against the
same seven-day minimum before release.

Use `cargo cooldown update` when refreshing `Cargo.lock`; it downgrades too-new
registry releases to the newest compatible versions that satisfy the cooldown.

## Benchmarks

Local hot-path benchmarks:

```bash
cargo bench
```

Real gRPC load test with [`ghz`](https://ghz.sh/):

```bash
cargo run --release --bin uuid-rs -- --listen 127.0.0.1:50051
ghz \
  --proto proto/uuid_service.proto \
  --call uuid.v1.Uuid.Generate \
  --data '{}' \
  --duration 30s \
  --concurrency 64 \
  --insecure \
  127.0.0.1:50051
```

Record the machine, Rust version, command, concurrency, request rate, latency
percentiles, and throughput with any published result.

`*` Benchmark claims are local, reproducible, and constrained to machines and
commands that were actually run. Please do not summon the distributed systems
committee.
```

- [ ] **Step 3: Write dependency cooldown and quarantine config**

Create `cooldown.toml`:

```toml
[cooldown]
incompatible-publish-age = "deny"
lockfile-baseline = "ignore"

[registry]
global-min-publish-age = "7 days"
```

Create `quarantine.toml`:

```toml
[rules]
min_age_days = 7
git_min_age_days = 14
external_path = "deny"
```

- [ ] **Step 4: Write `prek` configuration**

Create `.pre-commit-config.yaml`:

```yaml
repos:
  - repo: local
    hooks:
      - id: cargo-fmt
        name: cargo fmt
        entry: cargo fmt --check
        language: system
        pass_filenames: false
      - id: cargo-clippy
        name: cargo clippy
        entry: cargo clippy --all-targets --all-features -- -D warnings
        language: system
        pass_filenames: false
      - id: cargo-test
        name: cargo test
        entry: cargo test
        language: system
        pass_filenames: false
      - id: cargo-doc
        name: cargo doc
        entry: cargo doc --no-deps
        language: system
        pass_filenames: false
      - id: dependency-age
        name: dependency age
        entry: cargo quarantine
        language: system
        pass_filenames: false
      - id: cargo-package
        name: cargo package
        entry: cargo package
        language: system
        pass_filenames: false
```

- [ ] **Step 5: Run benchmark compilation, dependency age, and docs checks**

Run:

```bash
cargo bench --no-run
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps
cargo quarantine
cargo package
```

Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add Cargo.toml benches/uuid_hot_path.rs README.md cooldown.toml quarantine.toml .pre-commit-config.yaml
git commit -m "docs: add benchmarks and release checks"
```

---

### Task 6: Final Verification

**Files:**
- Modify as needed only if verification exposes issues.

- [ ] **Step 1: Update local stable toolchain**

Run:

```bash
rustup update stable
rustc --version
cargo --version
```

Expected: Rust reports stable `1.96.0` or newer.

- [ ] **Step 2: Run full local verification**

Run:

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps
cargo bench --no-run
cargo quarantine
cargo package
```

Expected: all commands pass.

- [ ] **Step 3: Optionally run `prek`**

Run:

```bash
prek run --all-files
```

Expected: all hooks pass. If `prek` is not installed, report that the hook config exists and the equivalent cargo commands passed.

- [ ] **Step 4: Commit final verification fixes if needed**

```bash
git add .
git commit -m "chore: finalize uuid service"
```
