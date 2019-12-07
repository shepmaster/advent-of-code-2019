pub type Error = Box<dyn std::error::Error>;
pub type Result<T, E = Error> = std::result::Result<T, E>;

fn amplifier(program: &intcode::Program, phase: &[intcode::Byte]) -> Result<intcode::Byte> {
    let mut signal_value = 0;

    for &phase_digit in phase {
        let mut amp_program = program.to_owned();
        let input = vec![phase_digit, signal_value];
        let output = intcode::execute(&mut amp_program, input)?;
        signal_value = *output.last().expect("Amplifier did not produce output");
    }

    Ok(signal_value)
}

fn search_for_max(program: &intcode::Program) -> Result<Option<intcode::Byte>> {
    let mut phases = [0, 1, 2, 3, 4];
    let heap = permutohedron::Heap::new(&mut phases);

    itertools::process_results(
        heap.into_iter().map(|phase| amplifier(program, &phase)),
        |i| i.max(),
    )
}

#[cfg(test)]
mod test {
    use super::*;

    const EXAMPLE_PROGRAM_1: &[intcode::Byte] = &[
        3, 15, 3, 16, 1002, 16, 10, 16, 1, 16, 15, 15, 4, 15, 99, 0, 0,
    ];

    const EXAMPLE_PROGRAM_2: &[intcode::Byte] = &[
        3, 23, 3, 24, 1002, 24, 10, 24, 1002, 23, -1, 23, 101, 5, 23, 23, 1, 24, 23, 23, 4, 23, 99,
        0, 0,
    ];

    const EXAMPLE_PROGRAM_3: &[intcode::Byte] = &[
        3, 31, 3, 32, 1002, 32, 10, 32, 1001, 31, -2, 31, 1007, 31, 0, 33, 1002, 33, 7, 33, 1, 33,
        31, 31, 1, 32, 31, 31, 4, 31, 99, 0, 0, 0,
    ];

    #[test]
    fn amplifier_functionality() -> Result<()> {
        assert_eq!(amplifier(EXAMPLE_PROGRAM_1, &[4, 3, 2, 1, 0])?, 43210);

        assert_eq!(amplifier(EXAMPLE_PROGRAM_2, &[0, 1, 2, 3, 4])?, 54321);

        assert_eq!(amplifier(EXAMPLE_PROGRAM_3, &[1, 0, 4, 3, 2])?, 65210);

        Ok(())
    }

    #[test]
    fn search_functionality() -> Result<()> {
        assert_eq!(search_for_max(EXAMPLE_PROGRAM_1)?, Some(43210));

        assert_eq!(search_for_max(EXAMPLE_PROGRAM_2)?, Some(54321));

        assert_eq!(search_for_max(EXAMPLE_PROGRAM_3)?, Some(65210));

        Ok(())
    }
}

const INPUT: &str = include_str!("input.txt");

fn main() -> Result<()> {
    let program = INPUT
        .trim()
        .split(",")
        .map(str::parse)
        .collect::<Result<Vec<_>, _>>()?;

    println!("{:?}", search_for_max(&program));

    Ok(())
}
