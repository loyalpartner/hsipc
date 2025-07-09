//! Simplified performance benchmarks for hsipc
//!
//! These benchmarks validate the key performance claims in README.md

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use hsipc::ProcessHub;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use tokio::runtime::Runtime;

#[derive(Serialize, Deserialize, Clone)]
struct BenchmarkData {
    data: Vec<u8>,
    sequence: u64,
}

impl BenchmarkData {
    fn new(size: usize, sequence: u64) -> Self {
        Self {
            data: vec![0u8; size],
            sequence,
        }
    }
}

/// Test simple message throughput
fn benchmark_message_throughput(c: &mut Criterion) {
    let _rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("message_throughput");

    for size in [64, 256, 1024, 4096].iter() {
        group.throughput(Throughput::Bytes(*size as u64));

        group.bench_with_input(
            BenchmarkId::new("serialize_deserialize", size),
            size,
            |b, &size| {
                b.iter(|| {
                    let data = BenchmarkData::new(size, 0);
                    let serialized = bincode::serialize(&data).unwrap();
                    let deserialized: BenchmarkData = bincode::deserialize(&serialized).unwrap();
                    black_box(deserialized);
                });
            },
        );
    }

    group.finish();
}

/// Test ProcessHub creation and basic operations
fn benchmark_hub_operations(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("hub_operations");

    group.bench_function("hub_creation", |b| {
        b.iter(|| {
            let hub = rt.block_on(ProcessHub::new("bench_hub")).unwrap();
            black_box(hub);
        });
    });

    group.bench_function("event_publishing", |b| {
        let hub = rt.block_on(ProcessHub::new("bench_pub_hub")).unwrap();

        b.iter(|| {
            let data = BenchmarkData::new(64, 0);
            rt.block_on(hub.publish("benchmark/test", data)).unwrap();
        });
    });

    group.finish();
}

/// Test high-frequency operations
fn benchmark_high_frequency(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("high_frequency");
    group.measurement_time(Duration::from_secs(5));

    group.bench_function("rapid_serialization", |b| {
        b.iter(|| {
            for i in 0..1000 {
                let data = BenchmarkData::new(64, i);
                let serialized = bincode::serialize(&data).unwrap();
                black_box(serialized);
            }
        });
    });

    group.bench_function("rapid_publishing", |b| {
        let hub = rt.block_on(ProcessHub::new("bench_rapid_hub")).unwrap();

        b.iter(|| {
            rt.block_on(async {
                for i in 0..100 {
                    let data = BenchmarkData::new(64, i);
                    hub.publish("benchmark/rapid", data).await.unwrap();
                }
            });
        });
    });

    group.finish();
}

/// Test concurrent operations
fn benchmark_concurrent_operations(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("concurrent_operations");
    group.measurement_time(Duration::from_secs(10));

    for num_tasks in [1, 2, 4, 8].iter() {
        group.bench_with_input(
            BenchmarkId::new("concurrent_publishing", num_tasks),
            num_tasks,
            |b, &num_tasks| {
                b.iter(|| {
                    rt.block_on(async {
                        let hub = ProcessHub::new("bench_concurrent_hub").await.unwrap();
                        let mut handles = Vec::new();

                        for i in 0..num_tasks {
                            let hub_clone = hub.clone();
                            let handle = tokio::spawn(async move {
                                for j in 0..50 {
                                    let data = BenchmarkData::new(64, (i * 50 + j) as u64);
                                    hub_clone
                                        .publish("benchmark/concurrent", data)
                                        .await
                                        .unwrap();
                                }
                            });
                            handles.push(handle);
                        }

                        // Wait for all tasks to complete
                        for handle in handles {
                            handle.await.unwrap();
                        }
                    });
                });
            },
        );
    }

    group.finish();
}

/// Test latency with small messages
fn benchmark_latency(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();

    let mut group = c.benchmark_group("latency");
    group.measurement_time(Duration::from_secs(10));

    group.bench_function("small_message_latency", |b| {
        b.iter(|| {
            let start = std::time::Instant::now();

            let data = BenchmarkData::new(64, 0);
            let serialized = bincode::serialize(&data).unwrap();
            let _deserialized: BenchmarkData = bincode::deserialize(&serialized).unwrap();

            let duration = start.elapsed();
            black_box(duration);
        });
    });

    group.bench_function("hub_publish_latency", |b| {
        let hub = rt.block_on(ProcessHub::new("bench_latency_hub")).unwrap();

        b.iter(|| {
            let start = std::time::Instant::now();

            let data = BenchmarkData::new(64, 0);
            rt.block_on(hub.publish("benchmark/latency", data)).unwrap();

            let duration = start.elapsed();
            black_box(duration);
        });
    });

    group.finish();
}

criterion_group!(
    benches,
    benchmark_message_throughput,
    benchmark_hub_operations,
    benchmark_high_frequency,
    benchmark_concurrent_operations,
    benchmark_latency
);

criterion_main!(benches);
