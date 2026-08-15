use std::sync::{Arc, Mutex};
use std::thread;

fn shared_statuses() -> Arc<Mutex<Vec<u16>>> {
    Arc::new(Mutex::new(Vec::new()))
}

fn assert_send_sync<T: Send + Sync>() {}

fn main() {}

#[test]
fn state_can_move_and_be_shared_between_threads() {
    assert_send_sync::<Arc<Mutex<Vec<u16>>>>();
    let statuses = shared_statuses();
    let worker_view = Arc::clone(&statuses);
    thread::spawn(move || worker_view.lock().unwrap().push(200))
        .join()
        .unwrap();
    assert_eq!(*statuses.lock().unwrap(), vec![200]);
}
