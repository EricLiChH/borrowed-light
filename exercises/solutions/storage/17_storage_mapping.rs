enum Outcome {
    Reachable(u16),
    Unreachable { kind: String, reason: String },
}

fn columns(outcome: &Outcome) -> (Option<u16>, Option<&str>, Option<&str>) {
    match outcome {
        Outcome::Reachable(status) => (Some(*status), None, None),
        Outcome::Unreachable { kind, reason } => (None, Some(kind), Some(reason)),
    }
}

fn main() {}

#[test]
fn enum_variants_map_to_exclusive_columns() {
    assert_eq!(columns(&Outcome::Reachable(204)), (Some(204), None, None));
    let failed = Outcome::Unreachable {
        kind: "timeout".into(),
        reason: "deadline".into(),
    };
    assert_eq!(columns(&failed), (None, Some("timeout"), Some("deadline")));
}
