fn longer<'a>(left: &'a str, right: &'a str) -> &'a str {
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
