use itertools::Itertools;
use std::{collections::BTreeMap, convert::TryFrom};

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum Tile {
    /// No game object appears in this tile.
    Empty = 0,
    /// Walls are indestructible barriers.
    Wall = 1,
    /// Blocks can be broken by the ball.
    Block = 2,
    /// The paddle is indestructible.
    HorizontalPaddle = 3,
    ///  The ball moves diagonally and bounces off objects.
    Ball = 4,
}

impl TryFrom<intcode::Byte> for Tile {
    type Error = intcode::Byte;

    fn try_from(v: intcode::Byte) -> Result<Self, Self::Error> {
        use Tile::*;

        Ok(match v {
            v if v == Empty as intcode::Byte => Empty,
            v if v == Wall as intcode::Byte => Wall,
            v if v == Block as intcode::Byte => Block,
            v if v == HorizontalPaddle as intcode::Byte => HorizontalPaddle,
            v if v == Ball as intcode::Byte => Ball,
            o => return Err(o),
        })
    }
}

type Coord = (intcode::Byte, intcode::Byte);
type Board = BTreeMap<Coord, Tile>;

fn print_board(board: &Board) {
    let bounds = match board.keys().copied().collect() {
        util::BoundsCollect(Some(bounds)) => bounds,
        _ => return,
    };

    let xs = bounds.min_x..=bounds.max_x;
    let ys = bounds.min_y..=bounds.max_y;

    print!("\x1B[2J");

    for y in ys.clone() {
        for x in xs.clone() {
            use Tile::*;
            let c = match board.get(&(x, y)).copied().unwrap_or(Empty) {
                Empty => ' ',
                Wall => '█',
                Block => '▒',
                HorizontalPaddle => '━',
                Ball => '⦿',
            };
            print!("{}", c);
        }
        println!();
    }
}

fn setup_game(computer: &mut intcode::Computer) -> Board {
    computer.execute_side_by_side(|_tx, rx| {
        let mut board = Board::new();
        for (x, y, tile) in rx.into_iter().tuples() {
            let tile = Tile::try_from(tile).expect("invalid tile");
            board.insert((x, y), tile);
        }
        board
    })
}

const LEFT: intcode::Byte = -1;
const NEUTRAL: intcode::Byte = 0;
const RIGHT: intcode::Byte = 1;

fn play_game(computer: &mut intcode::Computer, mut board: Board) -> intcode::Byte {
    // Play for free
    computer.program[0] = 2;

    computer.execute_side_by_side(move |tx, rx| {
        let mut score = 0;

        // Ignore errors - if the computer is shut down, we don't care.
        let _ = tx.send(NEUTRAL);

        for (i, (x, y, tile)) in rx.into_iter().tuples().enumerate() {
            if x == -1 && y == 0 {
                score = tile;
            } else {
                let tile = Tile::try_from(tile).expect("invalid tile");
                board.insert((x, y), tile);

                if tile == Tile::Ball {
                    let ((px, _), _) = board
                        .iter()
                        .find(|&(_, &t)| t == Tile::HorizontalPaddle)
                        .expect("No paddle");

                    use std::cmp::Ordering::*;
                    let paddle_direction = match x.cmp(&px) {
                        Less => LEFT,
                        Equal => NEUTRAL,
                        Greater => RIGHT,
                    };

                    // Ignore errors - if the computer is shut down, we don't care.
                    let _ = tx.send(paddle_direction);

                    std::thread::sleep(std::time::Duration::from_millis(16));
                    print_board(&board);
                    println!("{}\t\t{}", i, score);
                }
            }
        }

        score
    })
}

const INPUT: &str = include_str!("input.txt");

fn main() {
    let mut computer: intcode::Computer = INPUT.parse().expect("Unable to parse program");

    let board = setup_game(&mut computer);
    let tiles = board.iter().filter(|&(_, &t)| t == Tile::Block).count();
    println!("{}", tiles);

    let score = play_game(&mut computer, board);
    println!("{}", score);
}
