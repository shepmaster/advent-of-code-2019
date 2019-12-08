use std::{iter, thread};

pub type Error = Box<dyn std::error::Error + Send + Sync + 'static>;
pub type Result<T, E = Error> = std::result::Result<T, E>;

fn amplifier(program: &intcode::Program, phase: &[intcode::Byte]) -> Result<intcode::Byte> {
    // T0R0 T1R1 T2R2 T3R3 T4R4
    let (mut txs, rxs): (Vec<_>, Vec<_>) = iter::repeat_with(intcode::channel)
        .take(phase.len())
        .unzip();

    let tx = txs.first().unwrap().clone();
    let rx = rxs.first().unwrap().clone();

    // Load phase while tx/rx are still in sync
    for (tx, &phase_digit) in txs.iter().zip(phase) {
        tx.send(phase_digit).expect("Unable to load phase digit");
    }

    // T1R0 T2R1 T3R2 T4R3 T0R4
    txs.rotate_left(1);

    // <T1R0> A0; <T2R1> A1; <T3R2> A2; <T4R3> A3; <T0R4> A4
    let amps: Vec<_> = txs
        .into_iter()
        .zip(rxs)
        .map(|(tx, rx)| {
            let mut program = program.to_owned();

            thread::spawn(move || intcode::execute_with_output(&mut program, rx, tx))
        })
        .collect();

    // Send initial value
    tx.send(0).map_err(Error::from)?;
    drop(tx);

    amps.into_iter()
        .map(|t| t.join().expect("Thread panicked"))
        .collect::<Result<Vec<()>, _>>()?;

    // Get last value
    rx.recv().map_err(Into::into)
}

enum SearchSpace {
    Plain,
    Feedback,
}

fn search_for_max(program: &intcode::Program, space: SearchSpace) -> Result<Option<intcode::Byte>> {
    let mut phases = match space {
        SearchSpace::Plain => [0, 1, 2, 3, 4],
        SearchSpace::Feedback => [5, 6, 7, 8, 9],
    };
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
        use SearchSpace::*;

        assert_eq!(search_for_max(EXAMPLE_PROGRAM_1, Plain)?, Some(43210));

        assert_eq!(search_for_max(EXAMPLE_PROGRAM_2, Plain)?, Some(54321));

        assert_eq!(search_for_max(EXAMPLE_PROGRAM_3, Plain)?, Some(65210));

        Ok(())
    }

    const FEEDBACK_PROGRAM_1: &[intcode::Byte] = &[
        3, 26, 1001, 26, -4, 26, 3, 27, 1002, 27, 2, 27, 1, 27, 26, 27, 4, 27, 1001, 28, -1, 28,
        1005, 28, 6, 99, 0, 0, 5,
    ];

    const FEEDBACK_PROGRAM_2: &[intcode::Byte] = &[
        3, 52, 1001, 52, -5, 52, 3, 53, 1, 52, 56, 54, 1007, 54, 5, 55, 1005, 55, 26, 1001, 54, -5,
        54, 1105, 1, 12, 1, 53, 54, 53, 1008, 54, 0, 55, 1001, 55, 1, 55, 2, 53, 55, 53, 4, 53,
        1001, 56, -1, 56, 1005, 56, 6, 99, 0, 0, 0, 0, 10,
    ];

    #[test]
    fn amplifier_feedback_functionality() -> Result<()> {
        assert_eq!(amplifier(FEEDBACK_PROGRAM_1, &[9, 8, 7, 6, 5])?, 139629729);

        assert_eq!(amplifier(FEEDBACK_PROGRAM_2, &[9, 7, 8, 5, 6])?, 18216);

        Ok(())
    }

    #[test]
    fn search_feedback_functionality() -> Result<()> {
        use SearchSpace::*;

        assert_eq!(
            search_for_max(FEEDBACK_PROGRAM_1, Feedback)?,
            Some(139629729)
        );

        assert_eq!(search_for_max(FEEDBACK_PROGRAM_2, Feedback)?, Some(18216));

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

    println!("{:?}", search_for_max(&program, SearchSpace::Plain));
    println!("{:?}", search_for_max(&program, SearchSpace::Feedback));

    Ok(())
}
