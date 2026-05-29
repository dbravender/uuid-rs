//! Hot-path benchmarks for UUID generation and client-side formatting.

use std::hint::black_box;

use criterion::{Criterion, criterion_group, criterion_main};
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
