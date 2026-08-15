use std::sync::mpsc;
use std::thread;

fn collect_statuses() -> Vec<u16> {
    let (sender, receiver) = mpsc::channel();
    let first = sender.clone();
    thread::spawn(move || first.send(204).expect("receiver should stay open"));
    thread::spawn(move || sender.send(200).expect("receiver should stay open"));
    let mut statuses = receiver.iter().collect::<Vec<_>>();
    statuses.sort_unstable();
    statuses
}

fn main() {}

#[test]
fn workers_transfer_statuses_through_a_channel() {
    assert_eq!(collect_statuses(), vec![200, 204]);
}
