use std::convert::TryInto;

fn unpack_binop_args(state: &mut [usize], pc: usize) -> (usize, usize, &mut usize) {
    let x = &state[pc + 1..][..3];
    let x: [_; 3] = x.try_into().expect("Not enough arguments for binary operation");
    let [l, r, o] = x;
    let l = state[l];
    let r = state[r];
    let o = &mut state[o];
    (l, r, o)
}

fn intcode(state: &mut [usize]) {
    let mut pc = 0;

    loop {
        match state[pc] {
            1 => {
                let (l, r, o) = unpack_binop_args(state, pc);
                *o = l + r;
            }
            2 => {
                let (l, r, o) = unpack_binop_args(state, pc);
                *o = l * r;
            }
            99 => return,
            _ => unreachable!("Not an opcode"),
        }
        pc += 4;
    }
}

#[test]
fn specifications() {
    let mut state = [1, 0, 0, 0, 99];
    intcode(&mut state);
    assert_eq!(state, [2, 0, 0, 0, 99]);

    let mut state = [2, 3, 0, 3, 99];
    intcode(&mut state);
    assert_eq!(state, [2, 3, 0, 6, 99]);

    let mut state = [2, 4, 4, 5, 99, 0];
    intcode(&mut state);
    assert_eq!(state, [2, 4, 4, 5, 99, 9801]);

    let mut state = [1, 1, 1, 4, 99, 5, 6, 0, 99];
    intcode(&mut state);
    assert_eq!(state, [30, 1, 1, 4, 2, 5, 6, 0, 99]);
}

const INPUT: &str = include_str!("input.txt");

fn main() {
    let mut input = INPUT
        .trim()
        .split(",")
        .map(str::parse)
        .collect::<Result<Vec<_>, _>>()
        .expect("Unable to load input");

    input[1] = 12;
    input[2] = 2;

    intcode(&mut input);
    println!("{}", input[0]);
}
