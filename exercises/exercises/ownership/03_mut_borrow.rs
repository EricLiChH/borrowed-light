#[derive(Debug)]
struct MonitorTarget {
    url: String,
}

fn normalize(target: &MonitorTarget) {
    // TODO: 让函数获得独占可变借用，并修改相应调用代码。
    if !target.url.starts_with("http://") && !target.url.starts_with("https://") {
        target.url.insert_str(0, "https://");
    }
}

fn main() {}

#[test]
fn normalization_adds_a_default_scheme() {
    let target = MonitorTarget {
        url: String::from("example.com"),
    };

    normalize(&target);

    assert_eq!(target.url, "https://example.com");
}
