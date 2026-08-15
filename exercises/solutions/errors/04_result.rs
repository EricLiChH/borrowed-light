fn parse_status(input: &str) -> Result<u16, String> {
    let status = input
        .parse::<u16>()
        .map_err(|error| format!("invalid status: {error}"))?;
    if (100..=599).contains(&status) {
        Ok(status)
    } else {
        Err(String::from("status must be between 100 and 599"))
    }
}

fn main() {}

#[test]
fn parses_and_validates_http_statuses() {
    assert_eq!(parse_status("204"), Ok(204));
    assert!(parse_status("not-a-number").is_err());
    assert!(parse_status("700").is_err());
}
