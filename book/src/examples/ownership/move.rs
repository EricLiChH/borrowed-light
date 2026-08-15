#[derive(Debug)]
struct MonitorTarget {
    url: String,
}

fn enqueue(target: MonitorTarget) -> Vec<MonitorTarget> {
    let mut queue = Vec::new();
    queue.push(target);
    queue
}

fn main() {
    let target = MonitorTarget {
        url: String::from("https://example.com"),
    };
    let queue = enqueue(target);
    println!("queued: {}", queue[0].url);
}
