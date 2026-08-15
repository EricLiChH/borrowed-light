use std::sync::mpsc;
use std::thread;

fn collect_statuses() -> Vec<u16> {
    // TODO: Spawn two workers, move cloned senders into them, then collect and sort.
    todo!()
}

fn main() {}

#[test]
fn workers_transfer_statuses_through_a_channel() {
    assert_eq!(collect_statuses(), vec![200, 204]);
}
