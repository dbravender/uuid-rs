# uuid-rs

```
    ██╗   ██╗██╗   ██╗██╗██████╗       ██████╗ ███████╗
    ██║   ██║██║   ██║██║██╔══██╗      ██╔══██╗██╔════╝
    ██║   ██║██║   ██║██║██║  ██║█████╗██████╔╝███████╗
    ██║   ██║██║   ██║██║██║  ██║╚════╝██╔══██╗╚════██║
    ╚██████╔╝╚██████╔╝██║██████╔╝      ██║  ██║███████║
     ╚═════╝  ╚═════╝ ╚═╝╚═════╝       ╚═╝  ╚═╝╚══════╝
   ·▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄▄·
   ·  T H E   W O R L D ' S   F A S T E S T   U U I D ·
   ·    M I C R O S E R V I C E *  ( I  T H I N K )   ·
   ·▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀▀·
              ▄▄▄  16 BYTES   ▄▄▄  173 CRATES
             ▐███▌ OVER THE  ▐███▌ TO SHIP
              ▀▀▀  WIRE       ▀▀▀   THEM
```

> The world's fastest uuid microservice*
> 
> * that I'm aware of

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

## Architecture

```
            ╔════════════════════════════════════════════════════════════╗
            ║         T H E   u u i d - r s   M I C R O S E R V I C E    ║
            ╚════════════════════════════════════════════════════════════╝

   ┌───────────────────────────┐                  ┌───────────────────────────┐
   │        C L I E N T        │                  │        S E R V E R        │
   │      (examples/client)    │                  │       (bin: uuid-rs)      │
   ├───────────────────────────┤                  ├───────────────────────────┤
   │                           │                  │                           │
   │  GenerateRequest {}       │                  │   ┌───────────────────┐   │
   │   (literally empty)       │                  │   │   UuidGenerator   │   │
   │           │               │                  │   │   ::generate()    │   │
   │           ▼               │                  │   └─────────┬─────────┘   │
   │  ┌─────────────────┐      │   ░░ HTTP/2 ░░   │             ▼             │
   │  │  tonic  client  │ ═══════════ TCP ══════════▶ ┌───────────────────┐   │
   │  │  hyper · h2     │ ◀══════ (the slow part) ═══ │  Uuid::new_v4()   │   │
   │  └─────────────────┘      │   ~tens of µs    │   │     ~744 ns       │   │
   │           │               │                  │   └─────────┬─────────┘   │
   │           ▼               │                  │             ▼             │
   │  16 raw bytes  ░░░░░░░    │                  │  GenerateResponse {       │
   │           │               │                  │    uuid: [u8; 16]         │
   │           ▼               │                  │  } ← 16 bytes, no alloc   │
   │  uuid.hyphenated()        │                  │     of string. ever.      │
   │     ~26 ns  ◀── the only  │                  │                           │
   │  67e55044-10b1-426f-...   │      part this   │                           │
   │             project saves │                  │                           │
   └───────────────────────────┘                  └───────────────────────────┘

   ┌──────────────────────────────────────────────────────────────────────────┐
   │  ⚠  PERF NOTE: the network round-trip costs ~1000x more than the UUID.   │
   │     We heroically avoided ~26 ns of string formatting on the server and  │
   │     then shipped the bytes across a ~50,000 ns gRPC stack to do it.      │
   │     This is the joke. The joke is the architecture.                      │
   └──────────────────────────────────────────────────────────────────────────┘

   DEPENDENCY ICEBERG (what it takes to move 16 bytes)
   ───────────────────────────────────────────────────
     uuid          ▏▏▏▏                                     4 crates  ← the job
     tokio         ▏▏▏▏▏▏▏▏▏▏▏▏▏                           13 crates
     clap          ▏▏▏▏▏▏▏▏▏▏▏▏▏▏▏▏▏                       17 crates
     tonic (gRPC)  ▏▏▏▏▏▏▏▏▏▏▏▏▏▏▏▏▏▏▏▏▏▏▏▏▏▏▏▏▏▏▏▏▏...    65 crates  ← the bit
                                                          ───────────
                                                          173 total
```

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

## Dependency Policy

The direct runtime dependencies are intentionally boring and widely used Rust
ecosystem crates: `tokio`, `tonic`, `prost`, `uuid`, and `clap`. Direct dependency
versions are exact-pinned to releases published at least seven days before adoption.

`Cargo.lock` is committed for the service binary. Use `cargo cooldown update` when
refreshing the lockfile, then verify the result with `cargo quarantine`.

## Supply Chain Security

CI follows the GitHub Actions hardening practices described in Astral's
[Open source security at Astral](https://astral.sh/blog/open-source-security-at-astral):

- **Actions pinned to commit SHAs.** Every `uses:` in
  [`.github/workflows/ci.yml`](.github/workflows/ci.yml) is pinned to a full-length
  commit SHA (with the human-readable version as a trailing comment), because tags
  and branches are mutable and can be repointed after review.
- **Minimal token permissions.** The workflow defaults to `permissions: {}` and each
  job opts into only what it needs (`contents: read` here).
- **No persisted credentials.** `actions/checkout` runs with
  `persist-credentials: false` so the `GITHUB_TOKEN` is not left in the checkout's
  git config for later steps to reuse.
- **Workflows audited with [`zizmor`](https://github.com/zizmorcore/zizmor).** It runs
  as part of the `prek` suite (locally and in CI) and flags unpinned actions,
  over-broad permissions, and impostor commits.

When bumping a pinned action, re-resolve the tag to its commit SHA rather than
trusting the tag in place.

## Quality Checks

The full check suite (format, clippy, tests, docs, dependency age, package dry
run) is declared once in [`.pre-commit-config.yaml`](.pre-commit-config.yaml) and
run with [`prek`](https://github.com/j178/prek).

```bash
prek install          # one-time: run the checks automatically on every git commit
prek run --all-files  # run the whole suite on demand
```

The individual commands, if you want to run one in isolation:

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test
RUSTDOCFLAGS="-D warnings" cargo doc --no-deps
cargo bench --no-run
cargo quarantine
cargo package
```

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
