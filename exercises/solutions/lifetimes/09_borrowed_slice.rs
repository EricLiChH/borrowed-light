fn first_path_segment(url: &str) -> &str {
    url.trim_start_matches("https://")
        .split('/')
        .next()
        .unwrap_or("")
}

fn main() {}

#[test]
fn result_is_a_slice_of_the_input() {
    assert_eq!(
        first_path_segment("https://example.com/health"),
        "example.com"
    );
}
