#[derive(Debug, PartialEq, Eq)]
struct Target {
    name: String,
}

trait TargetSource {
    fn latest(&self) -> Option<&Target>;
}

// TODO: 为 Vec<Target> 实现 TargetSource，不要 clone。

fn main() {}

#[test]
fn borrows_the_latest_target() {
    let targets = vec![
        Target {
            name: String::from("Rust"),
        },
        Target {
            name: String::from("Local"),
        },
    ];

    assert_eq!(
        targets.latest().map(|target| target.name.as_str()),
        Some("Local")
    );
    assert_eq!(targets.len(), 2);
}
