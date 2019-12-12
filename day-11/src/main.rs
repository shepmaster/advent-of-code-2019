use itertools::Itertools;
use std::{collections::BTreeMap, thread};

const BLACK: intcode::Byte = 0;
const WHITE: intcode::Byte = 1;

const TURN_LEFT: intcode::Byte = 0;

type Coord = (i32, i32);
type Hull = BTreeMap<Coord, intcode::Byte>;

#[derive(Debug, Copy, Clone)]
enum Direction {
    Up,
    Down,
    Left,
    Right,
}

impl Direction {
    fn turn(&self, v: intcode::Byte) -> Self {
        use Direction::*;

        match (self, v == TURN_LEFT) {
            (Up, true) => Left,
            (Up, false) => Right,
            (Right, true) => Up,
            (Right, false) => Down,
            (Down, true) => Right,
            (Down, false) => Left,
            (Left, true) => Down,
            (Left, false) => Up,
        }
    }

    fn move_forward_from(&self, coord: Coord) -> Coord {
        use Direction::*;
        let (x, y) = coord;

        match self {
            Up => (x, y + 1),
            Down => (x, y - 1),
            Left => (x - 1, y),
            Right => (x + 1, y),
        }
    }
}

fn painted_squares(program: intcode::Program) -> usize {
    paint_common(program, BLACK).len()
}

fn painted_hull(program: intcode::Program) -> Hull {
    paint_common(program, WHITE)
}

fn paint_common(mut program: intcode::Program, initial_square: intcode::Byte) -> Hull {
    let (tx, rx) = intcode::channel();
    let (tx2, rx2) = intcode::channel();

    let painter = thread::spawn(move || {
        use Direction::*;

        let mut hull = BTreeMap::new();
        let mut position = (0, 0);
        let mut direction = Up;

        hull.insert(position, initial_square);

        let mut rx = rx.into_iter().tuples();

        loop {
            let color = hull.get(&position).copied().unwrap_or(BLACK);
            tx2.send(color)
                .expect("Computer has unexpectedly shut down");

            match rx.next() {
                Some((color, turn_direction)) => {
                    hull.insert(position, color);
                    direction = direction.turn(turn_direction);
                    position = direction.move_forward_from(position);
                }
                None => break,
            }
        }

        hull
    });

    let computer = thread::spawn(move || intcode::execute_with_output(&mut program, rx2, tx));

    computer
        .join()
        .expect("Computer panicked")
        .expect("Execution failed");
    let hull = painter.join().expect("Painter panicked");

    hull
}

const INPUT: &str = include_str!("input.txt");

fn main() {
    let program: Vec<_> = INPUT.trim().split(",").flat_map(str::parse).collect();

    let painted_squares = painted_squares(program.clone());
    println!("{}", painted_squares);

    let painted_hull = painted_hull(program);
    display_hull(&painted_hull);
}

fn display_hull(hull: &Hull) {
    let (min_x, max_x) = hull
        .keys()
        .copied()
        .map(|(x, _)| x)
        .minmax()
        .into_option()
        .expect("Nothing painted");
    let (min_y, max_y) = hull
        .keys()
        .copied()
        .map(|(_, y)| y)
        .minmax()
        .into_option()
        .expect("Nothing painted");

    for y in (min_y..=max_y).rev() {
        for x in min_x..=max_x {
            match hull.get(&(x, y)) {
                Some(&WHITE) => print!("█"),
                _ => print!(" "),
            }
        }
        println!();
    }
}
