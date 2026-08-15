fn parse_status(input: &str) -> Result<u16, String> {
    // TODO: 用 ? 传播解析错误，并拒绝 HTTP 范围之外的状态码。
    let _ = input;
    todo!("return a validated status code")
}

fn main() {}

#[test]
fn parses_and_validates_http_statuses() {
    assert_eq!(parse_status("204"), Ok(204));
    assert!(parse_status("not-a-number").is_err());
    assert!(parse_status("700").is_err());
}
