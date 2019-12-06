fn mass_to_fuel(mass: u32) -> u32 {
    (mass / 3).saturating_sub(2)
}

#[test]
fn specifications() {
    assert_eq!(mass_to_fuel(12), 2);
    assert_eq!(mass_to_fuel(14), 2);
    assert_eq!(mass_to_fuel(1969), 654);
    assert_eq!(mass_to_fuel(100756), 33583);
}

fn mass_to_fuel_recursive(mass: u32) -> u32 {
    std::iter::successors(Some(mass), |&mass| {
        let f = mass_to_fuel(mass);
        if f > 0 {
            Some(f)
        } else {
            None
        }
    })
    .skip(1) // Ignore the initial mass
    .sum()
}

#[test]
fn specifications2() {
    assert_eq!(mass_to_fuel_recursive(14), 2);
    assert_eq!(mass_to_fuel_recursive(1969), 966);
    assert_eq!(mass_to_fuel_recursive(100756), 50346);
}

const INPUT: &str = include_str!("input.txt");

fn main() {
    let sum = INPUT
        .lines()
        .map(|l| l.parse().map(mass_to_fuel_recursive))
        .sum::<Result<u32, _>>()
        .expect("Unable to compute sum");

    println!("{}", sum);
}
