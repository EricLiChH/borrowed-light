fn successful_count(statuses: &[u16]) -> usize {
    statuses
        .iter()
        .copied()
        .filter(|status| (200..300).contains(status))
        .count()
}

fn main() {}

#[test]
fn counts_successes_without_changing_the_input() {
    let statuses = [200, 204, 301, 404, 503];
    assert_eq!(successful_count(&statuses), 2);
    assert_eq!(statuses, [200, 204, 301, 404, 503]);
}
