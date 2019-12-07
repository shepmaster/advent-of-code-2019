use itertools::Itertools;

const INPUT: &str = include_str!("input.txt");

fn main() {
    let input = INPUT
        .trim()
        .split(",")
        .map(str::parse)
        .collect::<Result<Vec<_>, _>>()
        .expect("Unable to load input");

    for (noun, verb) in (0..100).cartesian_product(0..100) {
        let mut modified = input.clone();

        modified[1] = noun;
        modified[2] = verb;

        intcode::execute(&mut modified, None).expect("Unable to run program");

        if modified[0] == 19690720 {
            println!("{}", 100 * noun + verb);
            return;
        }
    }

    eprintln!("Ran out of inputs!")
}
