use itertools::Itertools;
use std::{
    collections::{BTreeMap, BTreeSet},
    convert::TryInto,
    str::FromStr,
};

type Error = Box<dyn std::error::Error>;
type Result<T, E = Error> = std::result::Result<T, E>;
type Dim = i32;
type Coord = (Dim, Dim);

struct Map(BTreeSet<Coord>);

impl FromStr for Map {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut map = BTreeSet::new();
        for (y, line) in s.trim().lines().enumerate() {
            for (x, point) in line.trim().chars().enumerate() {
                if point == '#' {
                    map.insert((x.try_into()?, y.try_into()?));
                }
            }
        }
        Ok(Map(map))
    }
}

impl Map {
    fn maximum_visible(&self) -> Result<(Coord, usize)> {
        let bounds = self.bounds()?;
        let mut max = None;

        for coord in self.0.iter().copied() {
            let n_visible = self.visible_at(bounds, coord)?;

            max = Some(match max {
                Some((c, n)) if n >= n_visible => (c, n),
                _ => (coord, n_visible),
            });
        }

        max.ok_or("No asteroids present").map_err(Into::into)
    }

    fn visible_at(&self, bounds: Bounds, coord: Coord) -> Result<usize> {
        let mut visible = BTreeMap::new();

        for other_asteroid in self.other_asteroids(coord) {
            // If we haven't visited this point before, assume we can
            // see it.
            visible.entry(other_asteroid).or_insert(true);

            for obscured_point in bounds.obscured_points_on_line(coord, other_asteroid) {
                visible.insert(obscured_point, false);
            }
        }

        Ok(visible.into_iter().filter(|&(_, visible)| visible).count())
    }

    fn bounds(&self) -> Result<Bounds> {
        let (min_x, max_x) = self
            .0
            .iter()
            .copied()
            .map(|(x, _)| x)
            .minmax()
            .into_option()
            .ok_or("Did not have any elements")?;

        let (min_y, max_y) = self
            .0
            .iter()
            .copied()
            .map(|(_, y)| y)
            .minmax()
            .into_option()
            .ok_or("Did not have any elements")?;

        Ok(Bounds {
            min_x,
            max_x,
            min_y,
            max_y,
        })
    }

    fn other_asteroids(&self, coord: Coord) -> impl Iterator<Item = Coord> + '_ {
        self.0.iter().filter(move |&&a| a != coord).copied()
    }
}

#[derive(Debug, Copy, Clone)]
struct Bounds {
    min_x: Dim,
    max_x: Dim,
    min_y: Dim,
    max_y: Dim,
}

impl Bounds {
    fn obscured_points_on_line(self, start: Coord, end: Coord) -> impl Iterator<Item = Coord> {
        let x_range = self.min_x..=self.max_x;
        let y_range = self.min_y..=self.max_y;

        let mut dx = end.0 - start.0;
        let mut dy = end.1 - start.1;

        let gcd = gcd(dx, dy);
        dx /= gcd;
        dy /= gcd;

        itertools::unfold(end, move |pt| {
            pt.0 += dx;
            pt.1 += dy;

            if x_range.contains(&pt.0) && y_range.contains(&pt.1) {
                Some(*pt)
            } else {
                None
            }
        })
    }
}

fn gcd(a: Dim, b: Dim) -> Dim {
    match (a.abs(), b.abs()) {
        (a, 0) => a,
        (a, b) => gcd(b, a % b),
    }
}

#[test]
fn gcd_negative() {
    assert_eq!(gcd(-5, 10), 5);
}

#[test]
fn bounds() -> Result<()> {
    let bounds = Bounds {
        min_x: 0,
        max_x: 5,
        min_y: 0,
        max_y: 5,
    };
    let points: Vec<_> = bounds.obscured_points_on_line((0, 0), (1, 1)).collect();
    assert_eq!(points, [(2, 2), (3, 3), (4, 4), (5, 5)]);
    Ok(())
}

#[test]
fn bounds_gcd() -> Result<()> {
    let bounds = Bounds {
        min_x: 0,
        max_x: 5,
        min_y: 0,
        max_y: 5,
    };

    // Positive direction
    let points: Vec<_> = bounds.obscured_points_on_line((0, 0), (2, 2)).collect();
    assert_eq!(points, [(3, 3), (4, 4), (5, 5)]);

    // Negative direction
    let points: Vec<_> = bounds.obscured_points_on_line((5, 5), (3, 3)).collect();
    assert_eq!(points, [(2, 2), (1, 1), (0, 0)]);

    Ok(())
}

#[test]
fn all_points() -> Result<()> {
    let map = r#"
        .#..#
        .....
        #####
        ....#
        ...##
    "#;

    let map: Map = map.parse()?;
    let bounds = map.bounds()?;

    let visible = r#"
        .7..7
        .....
        67775
        ....7
        ...87
    "#;

    let mut visible_map = BTreeMap::new();
    for (y, line) in visible.trim().lines().enumerate() {
        for (x, point) in line.trim().chars().enumerate() {
            if point != '.' {
                let coord: Coord = (x.try_into()?, y.try_into()?);
                let count: usize = point.to_digit(10).ok_or("Invalid digit")?.try_into()?;
                visible_map.insert(coord, count);
            }
        }
    }

    for (coord, expected_visible) in visible_map {
        assert_eq!(
            map.visible_at(bounds, coord)?,
            expected_visible,
            "at coordinate {:?}",
            coord
        );
    }

    assert_eq!(map.maximum_visible()?, ((3, 4), 8));

    Ok(())
}

#[test]
fn other_maps() -> Result<()> {
    let map = r#"
        ......#.#.
        #..#.#....
        ..#######.
        .#.#.###..
        .#..#.....
        ..#....#.#
        #..#....#.
        .##.#..###
        ##...#..#.
        .#....####
    "#;
    let map: Map = map.parse()?;
    assert_eq!(map.maximum_visible()?, ((5, 8), 33));

    let map = r#"
        #.#...#.#.
        .###....#.
        .#....#...
        ##.#.#.#.#
        ....#.#.#.
        .##..###.#
        ..#...##..
        ..##....##
        ......#...
        .####.###.
    "#;
    let map: Map = map.parse()?;
    assert_eq!(map.maximum_visible()?, ((1, 2), 35));

    let map = r#"
        .#..#..###
        ####.###.#
        ....###.#.
        ..###.##.#
        ##.##.#.#.
        ....###..#
        ..#.#..#.#
        #..#.#.###
        .##...##.#
        .....#.#..
    "#;
    let map: Map = map.parse()?;
    assert_eq!(map.maximum_visible()?, ((6, 3), 41));

    let map = r#"
        .#..##.###...#######
        ##.############..##.
        .#.######.########.#
        .###.#######.####.#.
        #####.##.#.##.###.##
        ..#####..#.#########
        ####################
        #.####....###.#.#.##
        ##.#################
        #####.##.###..####..
        ..######..##.#######
        ####.##.####...##..#
        .#####..#.######.###
        ##...#.##########...
        #.##########.#######
        .####.#.###.###.#.##
        ....##.##.###..#####
        .#.#.###########.###
        #.#.#.#####.####.###
        ###.##.####.##.#..##
    "#;
    let map: Map = map.parse()?;
    assert_eq!(map.maximum_visible()?, ((11, 13), 210));

    Ok(())
}

const INPUT: &str = include_str!("input.txt");

fn main() -> Result<()> {
    let map: Map = INPUT.parse()?;
    println!("{:?}", map.maximum_visible()?);
    Ok(())
}
