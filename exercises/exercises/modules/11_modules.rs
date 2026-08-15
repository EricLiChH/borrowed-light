mod monitor {
    pub struct Target {
        name: String,
    }

    impl Target {
        // TODO: Keep the field private while exposing a validating constructor and getter.
    }
}

fn main() {}

#[test]
fn module_exposes_behavior_not_fields() {
    let target = monitor::Target::new("Rust").expect("name should be valid");
    assert_eq!(target.name(), "Rust");
    assert!(monitor::Target::new("  ").is_none());
}
