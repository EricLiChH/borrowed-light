use std::future::Future;
use std::task::{Context, Poll, Waker};

async fn status_after_wait(status: u16) -> u16 {
    // TODO: Return a value so this future becomes ready when polled.
    todo!()
}

fn poll_ready(future: impl Future<Output = u16>) -> u16 {
    let mut future = Box::pin(future);
    let mut context = Context::from_waker(Waker::noop());
    match future.as_mut().poll(&mut context) {
        Poll::Ready(value) => value,
        Poll::Pending => panic!("this exercise future should be immediately ready"),
    }
}

fn main() {}

#[test]
fn async_call_returns_a_future_that_poll_advances() {
    assert_eq!(poll_ready(status_after_wait(204)), 204);
}
