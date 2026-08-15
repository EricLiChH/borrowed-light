#[derive(Debug, PartialEq, Eq)]
struct Target {
    name: String,
}

trait TargetSource {
    fn latest(&self) -> Option<&Target>;
}

impl TargetSource for Vec<Target> {
    fn latest(&self) -> Option<&Target> {
        self.last()
    }
}

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
