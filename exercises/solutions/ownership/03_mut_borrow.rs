#[derive(Debug)]
struct MonitorTarget {
    url: String,
}

fn normalize(target: &mut MonitorTarget) {
    if !target.url.starts_with("http://") && !target.url.starts_with("https://") {
        target.url.insert_str(0, "https://");
    }
}

fn main() {}

#[test]
fn normalization_adds_a_default_scheme() {
    let mut target = MonitorTarget {
        url: String::from("example.com"),
    };

    normalize(&mut target);

    assert_eq!(target.url, "https://example.com");
}
