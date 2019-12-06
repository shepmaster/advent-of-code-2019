use itertools::Itertools;
use std::collections::BTreeSet;
use std::str::FromStr;

type Error = Box<dyn std::error::Error>;
type Result<T, E = Error> = std::result::Result<T, E>;

type Position = (i32, i32);

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum Direction {
    U(i32),
    D(i32),
    L(i32),
    R(i32),
}

use Direction::*;

impl Direction {
    fn line_from(&self, start: Position) -> Box<dyn Iterator<Item = Position>> {
        let (x, y) = start;

        match *self {
            U(d) => Box::new((0..d).map(|d| d + 1).map(move |d| (x, y + d))),
            D(d) => Box::new((0..d).map(|d| d + 1).map(move |d| (x, y - d))),
            R(d) => Box::new((0..d).map(|d| d + 1).map(move |d| (x + d, y))),
            L(d) => Box::new((0..d).map(|d| d + 1).map(move |d| (x - d, y))),
        }
    }
}

impl FromStr for Direction {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        let (c, v) = s.split_at(1);
        let v = v.parse()?;
        Ok(match c {
            "U" => U(v),
            "D" => D(v),
            "L" => L(v),
            "R" => R(v),
            x => Err(format!("{} is not a valid direction", x))?,
        })
    }
}

#[test]
fn parsing() {
    assert_eq!("R75".parse::<Direction>().unwrap(), R(75));
    assert_eq!("D30".parse::<Direction>().unwrap(), D(30));
    assert_eq!("U83".parse::<Direction>().unwrap(), U(83));
    assert_eq!("L12".parse::<Direction>().unwrap(), L(12));
}

fn points(directions: impl IntoIterator<Item = Direction>) -> BTreeSet<Position> {
    let mut position = (0, 0);
    let mut visited = BTreeSet::new();

    for direction in directions {
        for point in direction.line_from(position) {
            visited.insert(point);
            position = point;
        }
    }

    visited
}

fn all_intersections(
    wires: impl IntoIterator<Item = impl IntoIterator<Item = Direction>>,
) -> BTreeSet<Position> {
    wires
        .into_iter()
        .map(points)
        .fold1(|intersections, wires| &intersections & &wires)
        .expect("Must have more than one wire")
}

fn distance_from_origin(position: Position) -> i32 {
    let (x, y) = position;
    x.abs() + y.abs()
}

fn closest_intersection(
    wires: impl IntoIterator<Item = impl IntoIterator<Item = Direction>>,
) -> Option<i32> {
    all_intersections(wires)
        .into_iter()
        .map(distance_from_origin)
        .filter(|&d| d > 0)
        .min()
}

#[test]
fn distance() {
    assert_eq!(
        closest_intersection(vec![
            vec![R(8), U(5), L(5), D(3)],
            vec![U(7), R(6), D(4), L(4)],
        ]),
        Some(6),
    );

    assert_eq!(
        closest_intersection(vec![
            vec![R(75), D(30), R(83), U(83), L(12), D(49), R(71), U(7), L(72)],
            vec![U(62), R(66), U(55), R(34), D(71), R(55), D(58), R(83)],
        ]),
        Some(159)
    );

    assert_eq!(
        closest_intersection(vec![
            vec![R(98), U(47), R(26), D(63), R(33), U(87), L(62), D(20), R(33), U(53), R(51)],
            vec![U(98), R(91), D(20), R(16), D(67), R(40), U(7), R(15), U(6), R(7)],
        ]),
        Some(135),
    );
}

const INPUT: &str = include_str!("input.txt");

fn main() {
    let input: Result<Vec<Vec<Direction>>, _> = INPUT
        .trim()
        .lines()
        .map(|l| l.split(",").map(str::parse).collect())
        .collect();
    let input = input.expect("Unable to parse input");

    println!("{:?}", closest_intersection(input));
}
