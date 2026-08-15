struct Selection<'a> {
    name: &'a str,
}

fn select_first(names: &[String]) -> Option<Selection<'_>> {
    names.first().map(|name| Selection { name })
}

fn main() {}

#[test]
fn struct_borrows_from_the_collection() {
    let names = vec![String::from("Rust"), String::from("Axum")];
    let selected = select_first(&names).expect("a first item should exist");
    assert_eq!(selected.name, "Rust");
}
