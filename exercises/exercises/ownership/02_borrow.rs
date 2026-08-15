#[derive(Debug)]
struct MonitorTarget {
    name: String,
    url: String,
}

fn label(target: MonitorTarget) -> String {
    format!("{} -> {}", target.name, target.url)
}

fn main() {}

#[test]
fn reading_a_label_does_not_consume_the_target() {
    // TODO: 让 label 只借用目标，调用后 target 仍可使用。
    let target = MonitorTarget {
        name: String::from("Rust"),
        url: String::from("https://www.rust-lang.org"),
    };

    let rendered = label(target);

    assert_eq!(rendered, "Rust -> https://www.rust-lang.org");
    assert_eq!(target.name, "Rust");
}
