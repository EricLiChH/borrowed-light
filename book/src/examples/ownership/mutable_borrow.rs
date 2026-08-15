fn normalize(url: &mut String) {
    if !url.starts_with("http://") && !url.starts_with("https://") {
        url.insert_str(0, "https://");
    }
}

fn main() {
    let mut url = String::from("example.com");
    normalize(&mut url);
    println!("{url}");
}
