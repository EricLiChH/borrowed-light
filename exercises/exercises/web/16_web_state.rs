use std::sync::{Arc, RwLock};

#[derive(Clone)]
struct AppState {
    targets: Arc<RwLock<Vec<String>>>,
}

fn new_state() -> AppState {
    // TODO: Create state whose clones share one synchronized target list.
    todo!()
}

fn main() {}

#[test]
fn cloned_state_points_to_the_same_collection() {
    let first = new_state();
    let second = first.clone();
    first.targets.write().unwrap().push("Rust".into());
    assert_eq!(second.targets.read().unwrap().as_slice(), ["Rust"]);
}
