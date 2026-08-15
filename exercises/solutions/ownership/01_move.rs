#[derive(Debug, PartialEq, Eq)]
struct MonitorTarget {
    name: String,
}

fn into_queue(target: MonitorTarget) -> Vec<MonitorTarget> {
    println!("queued target: {}", target.name);
    vec![target]
}

fn main() {}

#[test]
fn queue_owns_the_target() {
    let target = MonitorTarget {
        name: String::from("Rust"),
    };

    let queue = into_queue(target);

    assert_eq!(queue[0].name, "Rust");
}
