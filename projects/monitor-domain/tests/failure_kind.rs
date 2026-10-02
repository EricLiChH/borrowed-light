use monitor_domain::{CheckFailureKind, ParseFailureKindError};

#[test]
fn stable_names_round_trip_through_parse() {
    for kind in CheckFailureKind::ALL {
        assert_eq!(kind.as_str().parse::<CheckFailureKind>(), Ok(kind));
        assert_eq!(kind.to_string(), kind.as_str());
    }
}

#[test]
fn unknown_names_are_rejected_instead_of_guessed() {
    assert_eq!(
        "teapot".parse::<CheckFailureKind>(),
        Err(ParseFailureKindError)
    );
}
