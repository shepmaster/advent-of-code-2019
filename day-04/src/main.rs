use itertools::Itertools;

type Password = u32;

fn digits(password: Password) -> impl Iterator<Item = u8> {
    password
        .to_string()
        .split("")
        .flat_map(str::parse)
        .collect::<Vec<_>>()
        .into_iter()
}

fn correct_length(password: Password) -> bool {
    digits(password).count() == 6
}

fn has_double(password: Password) -> bool {
    digits(password).tuple_windows().any(|(a, b)| a == b)
}

fn is_sorted(password: Password) -> bool {
    digits(password).tuple_windows().all(|(a, b)| b >= a)

    // unstable
    // digits(password).is_sorted()
}

fn valid_password(password: Password) -> bool {
    correct_length(password) && has_double(password) && is_sorted(password)
}

#[test]
fn specifications() {
    assert!(valid_password(111111));
    assert!(!valid_password(223450));
    assert!(!valid_password(123789));
}

#[test]
fn debug_failures() {
    assert!(!valid_password(125733));
}

const MIN: Password = 125730;
const MAX: Password = 579381;

fn main() {
    let possible = (MIN..=MAX).filter(|&p| valid_password(p)).count();
    println!("{} (of {})", possible, MAX - MIN);

    // Wrong: 39352
    // ... Was using `tuples` instead of `tuple_windows`
}
