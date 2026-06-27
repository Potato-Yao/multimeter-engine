use criterion::{Criterion, criterion_group, criterion_main};
use futures::{SinkExt, StreamExt};
use std::sync::Once;
use std::time::Instant;
use tokio::net::{TcpListener, TcpStream};
use tokio_util::codec::{Framed, LinesCodec};

static INIT: Once = Once::new();

const QUERY_TARGETS: &[&str] = &[
    "cpu_name",
    "cpu_usage",
    "cpu_temperature",
    "cpu_clock_rms",
    "gpu_name",
    "gpu_temperature",
    "mem_total",
    "mem_available",
    "bat_state",
    "os_name",
    "os_version",
    "os_host_name",
];

async fn start_test_server() -> String {
    INIT.call_once(|| {
        let _ = env_logger::builder().is_test(true).try_init();
        multimeter_engine::engine_init().unwrap();
    });

    let listener = TcpListener::bind(("127.0.0.1", 0)).await.unwrap();
    let addr = listener.local_addr().unwrap().to_string();

    tokio::spawn(async move {
        loop {
            let Ok((socket, _)) = listener.accept().await else {
                break;
            };

            tokio::spawn(async move {
                let mut framed = Framed::new(socket, LinesCodec::new());

                while let Some(Ok(line)) = framed.next().await {
                    let response = match multimeter_engine::web::handle_request(line) {
                        Ok(response) | Err(response) => response,
                    };
                    let response = serde_json::to_string(&response).unwrap();
                    framed.send(response).await.unwrap();
                }
            });
        }
    });

    addr
}

async fn connect(addr: &str) -> Framed<TcpStream, LinesCodec> {
    let socket = TcpStream::connect(addr).await.unwrap();
    Framed::new(socket, LinesCodec::new())
}

async fn query_once(framed: &mut Framed<TcpStream, LinesCodec>, target: &str) {
    let request = serde_json::json!({
        "version": 1,
        "id": format!("bench-{target}"),
        "kind": "get_info",
        "payload": {
            "value": target,
            "addition": null
        }
    });

    framed.send(request.to_string()).await.unwrap();

    let response = framed.next().await.unwrap().unwrap();
    let response: serde_json::Value = serde_json::from_str(&response).unwrap();

    assert_eq!(response["version"], 1);
    assert_eq!(response["id"], format!("bench-{target}"));
}

fn bench_tcp_get_info(c: &mut Criterion) {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let addr = runtime.block_on(start_test_server());

    c.bench_function("tcp get_info varied targets", |b| {
        b.to_async(&runtime).iter_custom(|iters| {
            let addr = addr.clone();

            async move {
                let mut framed = connect(&addr).await;
                let start = Instant::now();

                for i in 0..iters {
                    let target = QUERY_TARGETS[i as usize % QUERY_TARGETS.len()];
                    query_once(&mut framed, target).await;
                }

                start.elapsed()
            }
        });
    });
}

criterion_group!(benches, bench_tcp_get_info);
criterion_main!(benches);
