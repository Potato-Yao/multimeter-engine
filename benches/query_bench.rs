use criterion::{criterion_group, criterion_main, Criterion};
use multimeter_engine::monitor;
use multimeter_engine::monitor::{QueryRequest, QUERY_STATEMENTS};
use tokio::runtime::Runtime;

async fn run_concurrent_queries(n: usize) {
    let mut tasks = Vec::with_capacity(n);
    for target in QUERY_STATEMENTS.iter().take(n) {
        let target = target.to_string();
        tasks.push(async move {
            monitor::query_info(QueryRequest {
                target,
                parameter: None,
            })
        });
    }
    futures::future::join_all(tasks).await;
}

fn bench_queries(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    c.bench_function("concurrent_10_queries", |b| {
        b.to_async(&rt).iter(|| run_concurrent_queries(10));
    });
}

criterion_group!(benches, bench_queries);
criterion_main!(benches);
