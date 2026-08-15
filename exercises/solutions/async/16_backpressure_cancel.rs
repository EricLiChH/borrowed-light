use std::future::{Future, poll_fn};
use std::task::Poll;

use tokio::sync::mpsc;

async fn observe_backpressure_then_cancel() -> (bool, bool) {
    let (sender, mut receiver) = mpsc::channel(1);
    sender.send(1).await.expect("receiver should stay open");
    let mut blocked_send = Box::pin(sender.send(2));

    let backpressured = poll_fn(|context| match blocked_send.as_mut().poll(context) {
        Poll::Pending => Poll::Ready(true),
        Poll::Ready(_) => Poll::Ready(false),
    })
    .await;

    let shutdown = async {};
    let cancelled_by_shutdown = tokio::select! {
        biased;
        () = shutdown => true,
        result = &mut blocked_send => {
            let _ = result;
            false
        },
    };

    drop(blocked_send);
    let first_message = receiver.recv().await;
    let second_message_was_not_sent = receiver.try_recv().is_err();
    (
        backpressured,
        cancelled_by_shutdown && first_message == Some(1) && second_message_was_not_sent,
    )
}

fn main() {}

#[test]
fn full_channel_pauses_send_until_shutdown_cancels_it() {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .build()
        .expect("current-thread runtime should build");
    assert_eq!(
        runtime.block_on(observe_backpressure_then_cancel()),
        (true, true)
    );
}
