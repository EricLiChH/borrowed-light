#[derive(Debug)]
struct MonitorTarget {
    name: String,
    url: String,
}

fn label(target: &MonitorTarget) -> String {
    format!("{} -> {}", target.name, target.url)
}

fn main() {
    let target = MonitorTarget {
        name: String::from("Example"),
        url: String::from("https://example.com"),
    };
    println!("{}", label(&target));
    println!("still owned by main: {}", target.name);
}

