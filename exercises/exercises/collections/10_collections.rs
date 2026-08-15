use std::collections::HashMap;

fn summarize(statuses: &[u16]) -> HashMap<&'static str, usize> {
    // TODO: Count 2xx statuses as "healthy" and everything else as "other".
    todo!()
}

fn main() {}

#[test]
fn statuses_are_grouped() {
    let summary = summarize(&[200, 204, 404, 503]);
    assert_eq!(summary.get("healthy"), Some(&2));
    assert_eq!(summary.get("other"), Some(&2));
}
