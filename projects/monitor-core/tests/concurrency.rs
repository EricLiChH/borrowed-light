use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use monitor_core::{CheckPolicy, HealthChecker};
use monitor_domain::MonitorTarget;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::sync::{Notify, Semaphore};

#[tokio::test]
async fn learner_can_bound_batch_concurrency() {
    let probe = ConcurrencyProbe::start(4).await;
    let targets = (1..=4)
        .map(|number| {
            MonitorTarget::new(format!("local-{number}"), probe.url.clone())
                .expect("local URL should be valid")
        })
        .collect::<Vec<_>>();
    let policy = CheckPolicy::new(Duration::from_secs(1), 1, Duration::ZERO, 2)
        .expect("policy should be valid");
    let client = reqwest::Client::builder()
        .no_proxy()
        .build()
        .expect("test client should build");
    let checker = HealthChecker::new(client, policy);

    let batch = tokio::spawn(async move { checker.check_all(&targets).await });
    probe.wait_for_accepted(2).await;
    for _ in 0..4 {
        tokio::task::yield_now().await;
    }

    assert_eq!(probe.accepted.load(Ordering::SeqCst), 2);
    probe.release.add_permits(4);

    let results = batch.await.expect("batch task should finish");
    assert_eq!(results.len(), 4);
    assert!(probe.maximum.load(Ordering::SeqCst) <= 2);
}

struct ConcurrencyProbe {
    url: String,
    accepted: Arc<AtomicUsize>,
    maximum: Arc<AtomicUsize>,
    accepted_event: Arc<Notify>,
    release: Arc<Semaphore>,
}

impl ConcurrencyProbe {
    async fn start(expected_requests: usize) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("test server should bind");
        let address = listener
            .local_addr()
            .expect("listener should have an address");
        let accepted = Arc::new(AtomicUsize::new(0));
        let current = Arc::new(AtomicUsize::new(0));
        let maximum = Arc::new(AtomicUsize::new(0));
        let accepted_event = Arc::new(Notify::new());
        let release = Arc::new(Semaphore::new(0));

        let server_accepted = Arc::clone(&accepted);
        let server_current = Arc::clone(&current);
        let server_maximum = Arc::clone(&maximum);
        let server_event = Arc::clone(&accepted_event);
        let server_release = Arc::clone(&release);

        tokio::spawn(async move {
            for _ in 0..expected_requests {
                let (mut stream, _) = listener.accept().await.expect("server should accept");
                let handler_accepted = Arc::clone(&server_accepted);
                let handler_current = Arc::clone(&server_current);
                let handler_maximum = Arc::clone(&server_maximum);
                let handler_event = Arc::clone(&server_event);
                let handler_release = Arc::clone(&server_release);
                tokio::spawn(async move {
                    let mut request = [0_u8; 1024];
                    let _ = stream.read(&mut request).await;
                    handler_accepted.fetch_add(1, Ordering::SeqCst);
                    let now = handler_current.fetch_add(1, Ordering::SeqCst) + 1;
                    handler_maximum.fetch_max(now, Ordering::SeqCst);
                    handler_event.notify_waiters();
                    let permit = handler_release
                        .acquire()
                        .await
                        .expect("release semaphore should stay open");
                    permit.forget();
                    stream
                        .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 0\r\n\r\n")
                        .await
                        .expect("server should reply");
                    handler_current.fetch_sub(1, Ordering::SeqCst);
                });
            }
        });

        Self {
            url: format!("http://{address}"),
            accepted,
            maximum,
            accepted_event,
            release,
        }
    }

    async fn wait_for_accepted(&self, expected: usize) {
        while self.accepted.load(Ordering::SeqCst) < expected {
            self.accepted_event.notified().await;
        }
    }
}
