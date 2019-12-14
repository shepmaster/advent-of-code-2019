use itertools::Itertools;
use std::collections::BTreeMap;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum Tile {
    // /// No game object appears in this tile.
    // Empty = 0,
    // /// Walls are indestructible barriers.
    // Wall = 1,
    /// Blocks can be broken by the ball.
    Block = 2,
    // /// The paddle is indestructible.
    // HorizontalPaddle = 3,
    // ///  The ball moves diagonally and bounces off objects.
    // Ball = 4,
}

fn run_game(program: intcode::Program) -> usize {
    let board = intcode::execute_side_by_side(program, |_tx, rx| {
        let mut board = BTreeMap::new();
        for (x, y, tile) in rx.into_iter().tuples() {
            board.insert((x, y), tile);
        }
        board
    });

    board
        .into_iter()
        .filter(|&(_, t)| t == Tile::Block as intcode::Byte)
        .count()
}

const INPUT: &str = include_str!("input.txt");

fn main() {
    let program = intcode::parse_program(INPUT);
    println!("{}", run_game(program));
}
