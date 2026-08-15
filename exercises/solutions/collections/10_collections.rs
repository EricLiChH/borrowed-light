use std::collections::HashMap;

fn summarize(statuses: &[u16]) -> HashMap<&'static str, usize> {
    let mut summary = HashMap::new();
    for status in statuses {
        let label = if (200..300).contains(status) {
            "healthy"
        } else {
            "other"
        };
        *summary.entry(label).or_insert(0) += 1;
    }
    summary
}

fn main() {}

#[test]
fn statuses_are_grouped() {
    let summary = summarize(&[200, 204, 404, 503]);
    assert_eq!(summary.get("healthy"), Some(&2));
    assert_eq!(summary.get("other"), Some(&2));
}
