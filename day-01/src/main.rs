fn mass_to_fuel(mass: u32) -> u32 {
    (mass / 3) - 2
}

#[test]
fn specifications() {
    assert_eq!(mass_to_fuel(12), 2);
    assert_eq!(mass_to_fuel(14), 2);
    assert_eq!(mass_to_fuel(1969), 654);
    assert_eq!(mass_to_fuel(100756), 33583);
}

const INPUT: &str = include_str!("input.txt");

fn main() {
    let sum = INPUT
        .lines()
        .map(|l| l.parse().map(mass_to_fuel))
        .sum::<Result<u32, _>>()
        .expect("Unable to compute sum");

    println!("{}", sum);
}
