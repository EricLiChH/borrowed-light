fn longer(left: &str, right: &str) -> &str {
    // TODO: Add the relationship that lets the return value borrow either input.
    if left.len() >= right.len() {
        left
    } else {
        right
    }
}

fn main() {}

#[test]
fn return_borrows_from_the_inputs() {
    let owned = String::from("monitor");
    assert_eq!(longer(&owned, "web"), "monitor");
}
