#[derive(Debug, PartialEq, Eq)]
struct MonitorTarget {
    name: String,
}

fn into_queue(target: MonitorTarget) -> Vec<MonitorTarget> {
    // TODO: 修复移动后的使用，不要 clone。
    let queue = vec![target];
    println!("queued target: {}", target.name);
    queue
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
