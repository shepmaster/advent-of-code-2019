use itertools::Itertools;
use std::collections::BTreeMap;
use std::str::FromStr;

type Error = Box<dyn std::error::Error>;
type Result<T, E = Error> = std::result::Result<T, E>;

type Coordinate = (i32, i32);
type Distance = i32;

#[derive(Debug, Copy, Clone, Default, PartialEq, Eq)]
struct Position {
    coord: Coordinate,
    dist: Distance,
}

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
        let Position {
            coord: (x, y),
            dist,
        } = start;

        match *self {
            U(d) => Box::new((0..d).map(|d| d + 1).map(move |d| Position {
                coord: (x, y + d),
                dist: dist + d,
            })),
            D(d) => Box::new((0..d).map(|d| d + 1).map(move |d| Position {
                coord: (x, y - d),
                dist: dist + d,
            })),
            R(d) => Box::new((0..d).map(|d| d + 1).map(move |d| Position {
                coord: (x + d, y),
                dist: dist + d,
            })),
            L(d) => Box::new((0..d).map(|d| d + 1).map(move |d| Position {
                coord: (x - d, y),
                dist: dist + d,
            })),
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

fn points(directions: impl IntoIterator<Item = Direction>) -> BTreeMap<Coordinate, Distance> {
    let mut position = Position::default();
    let mut visited = BTreeMap::new();

    for direction in directions {
        for pos in direction.line_from(position) {
            let Position { coord, dist } = pos;
            // The first entry will have a minimal value
            visited.entry(coord).or_insert(dist);
            position = pos;
        }
    }

    visited
}

fn all_intersections(
    wires: impl IntoIterator<Item = impl IntoIterator<Item = Direction>>,
) -> BTreeMap<Coordinate, Distance> {
    wires
        .into_iter()
        .map(points)
        .fold1(|intersections, points| {
            intersections
                .into_iter()
                .flat_map(|(c, d1)| points.get(&c).map(|d2| (c, d1 + d2)))
                .collect()
        })
        .expect("Must have more than one wire")
}

fn closest_intersection(
    wires: impl IntoIterator<Item = impl IntoIterator<Item = Direction>>,
) -> Option<i32> {
    all_intersections(wires)
        .into_iter()
        .map(|(_, dist)| dist)
        .min()
}

#[test]
fn distance() {
    assert_eq!(
        closest_intersection(vec![
            vec![R(8), U(5), L(5), D(3)],
            vec![U(7), R(6), D(4), L(4)],
        ]),
        Some(30),
    );

    assert_eq!(
        closest_intersection(vec![
            vec![R(75), D(30), R(83), U(83), L(12), D(49), R(71), U(7), L(72)],
            vec![U(62), R(66), U(55), R(34), D(71), R(55), D(58), R(83)],
        ]),
        Some(610)
    );

    assert_eq!(
        closest_intersection(vec![
            vec![
                R(98),
                U(47),
                R(26),
                D(63),
                R(33),
                U(87),
                L(62),
                D(20),
                R(33),
                U(53),
                R(51)
            ],
            vec![
                U(98),
                R(91),
                D(20),
                R(16),
                D(67),
                R(40),
                U(7),
                R(15),
                U(6),
                R(7)
            ],
        ]),
        Some(410),
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
