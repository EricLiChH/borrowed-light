//! Fixtures shared by the monitor-core integration tests.
//!
//! Every integration test file compiles its own copy of this module, so a
//! helper used by only one file would otherwise be reported as dead code.
#![allow(dead_code)]

use std::num::NonZeroUsize;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::sync::{Notify, Semaphore};

/// Converts a literal into the non-zero type the policy demands.
pub fn nonzero(value: usize) -> NonZeroUsize {
    NonZeroUsize::new(value).expect("test literals must be non-zero")
}

/// Builds an HTTP client that never consults a proxy.
///
/// Tests talk to `127.0.0.1`, and a developer machine with a global proxy
/// configured would otherwise send those requests somewhere else entirely.
pub fn client() -> reqwest::Client {
    reqwest::Client::builder()
        .no_proxy()
        .build()
        .expect("test client should build")
}

/// Serves exactly one response, then stops.
pub async fn serve_once(response: &'static str) -> String {
    serve_raw(vec![response.to_owned()]).await.0
}

/// Serves one response after waiting, used to force out-of-order completion.
pub async fn serve_after(delay: Duration) -> String {
    let listener = bind().await;
    let address = local_address(&listener);
    tokio::spawn(async move {
        let (mut stream, _) = listener.accept().await.expect("server should accept");
        let mut request = [0_u8; 1024];
        let _ = stream.read(&mut request).await;
        tokio::time::sleep(delay).await;
        let _ = stream
            .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 0\r\n\r\n")
            .await;
    });
    format!("http://{address}")
}

/// Closes the connection without replying once, then answers `200 OK`.
pub async fn serve_failure_then_success() -> (String, Arc<AtomicUsize>) {
    let listener = bind().await;
    let address = local_address(&listener);
    let requests = Arc::new(AtomicUsize::new(0));
    let observed = Arc::clone(&requests);

    tokio::spawn(async move {
        for attempt in 1..=2 {
            let (mut stream, _) = listener.accept().await.expect("server should accept");
            observed.fetch_add(1, Ordering::SeqCst);
            let mut request = [0_u8; 1024];
            let _ = stream.read(&mut request).await;
            if attempt == 2 {
                let _ = stream
                    .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 0\r\n\r\n")
                    .await;
            }
        }
    });

    (format!("http://{address}"), requests)
}

/// Serves a fixed sequence of status codes, one per accepted connection.
pub async fn serve_status_sequence(statuses: &'static [u16]) -> (String, Arc<AtomicUsize>) {
    let listener = bind().await;
    let address = local_address(&listener);
    let requests = Arc::new(AtomicUsize::new(0));
    let observed = Arc::clone(&requests);

    tokio::spawn(async move {
        for status in statuses {
            let Ok((mut stream, _)) = listener.accept().await else {
                break;
            };
            observed.fetch_add(1, Ordering::SeqCst);
            let mut request = [0_u8; 1024];
            let _ = stream.read(&mut request).await;
            let response =
                format!("HTTP/1.1 {status}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n");
            let _ = stream.write_all(response.as_bytes()).await;
        }
    });

    (format!("http://{address}"), requests)
}

/// Replies `429 Too Many Requests` with `Retry-After`, then `200 OK`.
pub async fn serve_retry_after(seconds: u64) -> String {
    let listener = bind().await;
    let address = local_address(&listener);
    tokio::spawn(async move {
        for attempt in 1..=2_u8 {
            let Ok((mut stream, _)) = listener.accept().await else {
                break;
            };
            let mut request = [0_u8; 1024];
            let _ = stream.read(&mut request).await;
            let response = if attempt == 1 {
                format!(
                    "HTTP/1.1 429 Too Many Requests\r\nRetry-After: {seconds}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
                )
            } else {
                String::from("HTTP/1.1 200 OK\r\nContent-Length: 0\r\nConnection: close\r\n\r\n")
            };
            let _ = stream.write_all(response.as_bytes()).await;
        }
    });
    format!("http://{address}")
}

/// Redirects to itself forever, so reqwest gives up at its own redirect limit.
pub async fn serve_redirect_loop() -> (String, Arc<AtomicUsize>) {
    let listener = bind().await;
    let address = local_address(&listener);
    let url = format!("http://{address}/loop");
    let requests = Arc::new(AtomicUsize::new(0));
    let observed = Arc::clone(&requests);
    let location = url.clone();

    tokio::spawn(async move {
        for _ in 0..32 {
            let Ok((mut stream, _)) = listener.accept().await else {
                break;
            };
            observed.fetch_add(1, Ordering::SeqCst);
            let mut request = [0_u8; 1024];
            let _ = stream.read(&mut request).await;
            let response = format!(
                "HTTP/1.1 302 Found\r\nLocation: {location}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
            );
            let _ = stream.write_all(response.as_bytes()).await;
        }
    });

    (url, requests)
}

/// Serves the given raw responses in order, one per accepted connection.
async fn serve_raw(responses: Vec<String>) -> (String, Arc<AtomicUsize>) {
    let listener = bind().await;
    let address = local_address(&listener);
    let requests = Arc::new(AtomicUsize::new(0));
    let observed = Arc::clone(&requests);

    tokio::spawn(async move {
        for response in responses {
            let Ok((mut stream, _)) = listener.accept().await else {
                break;
            };
            observed.fetch_add(1, Ordering::SeqCst);
            let mut request = [0_u8; 1024];
            let _ = stream.read(&mut request).await;
            let _ = stream.write_all(response.as_bytes()).await;
        }
    });

    (format!("http://{address}"), requests)
}

async fn bind() -> TcpListener {
    TcpListener::bind("127.0.0.1:0")
        .await
        .expect("test server should bind")
}

fn local_address(listener: &TcpListener) -> std::net::SocketAddr {
    listener
        .local_addr()
        .expect("listener should have an address")
}

/// A server that accepts a fixed number of requests and holds each one open
/// until the test releases it, so the test can observe how many are in flight.
pub struct ConcurrencyProbe {
    pub url: String,
    pub accepted: Arc<AtomicUsize>,
    pub maximum: Arc<AtomicUsize>,
    accepted_event: Arc<Notify>,
    release: Arc<Semaphore>,
}

impl ConcurrencyProbe {
    /// Starts a server that expects the given number of connections.
    pub async fn start(expected_requests: usize) -> Self {
        let listener = bind().await;
        let address = local_address(&listener);
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
                    let _ = stream
                        .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 0\r\n\r\n")
                        .await;
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

    /// Waits until at least the given number of requests have arrived.
    pub async fn wait_for_accepted(&self, expected: usize) {
        while self.accepted.load(Ordering::SeqCst) < expected {
            self.accepted_event.notified().await;
        }
    }

    /// Lets every blocked handler answer.
    pub fn release_all(&self) {
        self.release.add_permits(64);
    }
}
