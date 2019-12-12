use std::{
    cmp::Ordering,
    collections::{BTreeMap, BTreeSet, VecDeque},
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
        let mut max = None;

        for coord in self.0.iter().copied() {
            let n_visible = self.visible_at(coord);

            max = Some(match max {
                Some((c, n)) if n >= n_visible => (c, n),
                _ => (coord, n_visible),
            });
        }

        max.ok_or("No asteroids present").map_err(Into::into)
    }

    fn visible_at(&self, coord: Coord) -> usize {
        self.lines_of_sight(coord).len()
    }

    fn vaporization_order(&self, coord: Coord) -> impl Iterator<Item = Coord> {
        let mut lines: VecDeque<_> = self
            .lines_of_sight(coord)
            .into_iter()
            .map(|(_, mut line)| {
                line.sort_by_key(|a| a.manhattan_distance());
                // Want the closest values at the end of the array for
                // easier popping
                line.reverse();
                line
            })
            .collect();

        itertools::unfold((), move |()| {
            let mut line = lines.pop_front()?;
            let next = line.pop()?;

            if !line.is_empty() {
                lines.push_back(line);
            }

            Some(next.apply_to(coord))
        })
    }

    fn lines_of_sight(&self, coord: Coord) -> BTreeMap<Angle, Vec<Angle>> {
        let mut lines = BTreeMap::new();

        for other_asteroid in self.other_asteroids(coord) {
            let a = Angle::from_coords(coord, other_asteroid);
            lines.entry(a.reduce()).or_insert_with(Vec::new).push(a);
        }

        lines
    }

    fn other_asteroids(&self, coord: Coord) -> impl Iterator<Item = Coord> + '_ {
        self.0.iter().filter(move |&&a| a != coord).copied()
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
struct Angle {
    dx: i32,
    dy: i32,
}

impl Angle {
    fn from_coords(start: Coord, end: Coord) -> Self {
        let dx = end.0 - start.0;
        let dy = end.1 - start.1;
        Self { dx, dy }
    }

    fn apply_to(self, coord: Coord) -> Coord {
        (coord.0 + self.dx, coord.1 + self.dy)
    }

    fn reduce(self) -> Self {
        let Self { mut dx, mut dy } = self;
        let gcd = gcd(dx, dy);
        dx /= gcd;
        dy /= gcd;
        Self { dx, dy }
    }

    fn manhattan_distance(self) -> i32 {
        self.dx.abs() + self.dy.abs()
    }

    fn as_decimal(self) -> f64 {
        use std::f64::consts::*;
        const TWO_PI: f64 = PI + PI;

        let dx = f64::from(self.dx);
        let dy = f64::from(self.dy);

        // Our Y axis grows downward
        let math_radians = f64::atan2(-dy, dx);

        // Differences from the pure math value:
        //
        // - Increasing is clockwise (negate)
        // - Zero degrees is straight up (add pi/2)
        // - Range from from 0 to 2pi (add 2pi and modulo)
        (-math_radians + FRAC_PI_2 + TWO_PI) % TWO_PI
    }
}

impl PartialOrd for Angle {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

// This will naturally sort by the angle the laser will take
impl Ord for Angle {
    fn cmp(&self, other: &Self) -> Ordering {
        self.as_decimal()
            .partial_cmp(&other.as_decimal())
            .expect("NaN unexpected")
    }
}

fn gcd(a: Dim, b: Dim) -> Dim {
    match (a.abs(), b.abs()) {
        (a, 0) => a,
        (a, b) => gcd(b, a % b),
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn angle_degrees() {
        use std::f64::consts::*;

        // Up
        assert_eq!(Angle { dx: 0, dy: -10 }.as_decimal(), 0.0);
        // Right
        assert_eq!(Angle { dx: 10, dy: 0 }.as_decimal(), FRAC_PI_2);
        // Down
        assert_eq!(Angle { dx: 0, dy: 10 }.as_decimal(), PI);
        // Left
        assert_eq!(Angle { dx: -10, dy: 0 }.as_decimal(), PI + FRAC_PI_2);
    }

    #[test]
    fn gcd_negative() {
        assert_eq!(gcd(-5, 10), 5);
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
                map.visible_at(coord),
                expected_visible,
                "at coordinate {:?}",
                coord
            );
        }

        assert_eq!(map.maximum_visible()?, ((3, 4), 8));

        Ok(())
    }

    const LARGE_MAP: &str = r#"
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

        let map: Map = LARGE_MAP.parse()?;
        assert_eq!(map.maximum_visible()?, ((11, 13), 210));

        Ok(())
    }

    #[test]
    fn vaporization() -> Result<()> {
        let map = r#"
            .#....#####...#..
            ##...##.#####..##
            ##...#...#.#####.
            ..#.....#...###..
            ..#.#.....#....##
        "#;
        let map: Map = map.parse()?;

        let order: Vec<_> = map.vaporization_order((8, 3)).take(9).collect();
        assert_eq!(order[0], (8, 1));
        assert_eq!(order[1], (9, 0));
        assert_eq!(order[2], (9, 1));
        assert_eq!(order[3], (10, 0));
        assert_eq!(order[4], (9, 2));
        assert_eq!(order[5], (11, 1));
        assert_eq!(order[6], (12, 1));
        assert_eq!(order[7], (11, 2));
        assert_eq!(order[8], (15, 1));

        let map: Map = LARGE_MAP.parse()?;

        let order: Vec<_> = map.vaporization_order((11, 13)).collect();
        assert_eq!(order[0], (11, 12));
        assert_eq!(order[1], (12, 1));
        assert_eq!(order[2], (12, 2));
        assert_eq!(order[9], (12, 8));
        assert_eq!(order[19], (16, 0));
        assert_eq!(order[49], (16, 9));
        assert_eq!(order[99], (10, 16));
        assert_eq!(order[198], (9, 6));
        assert_eq!(order[199], (8, 2));
        assert_eq!(order[200], (10, 9));
        assert_eq!(order[298], (11, 1));

        Ok(())
    }
}

const INPUT: &str = include_str!("input.txt");

fn main() -> Result<()> {
    let map: Map = INPUT.parse()?;
    let (station_coord, _) = map.maximum_visible()?;

    let asteroid_coord = map
        .vaporization_order(station_coord)
        .nth(199)
        .ok_or("Not enough vaporized")?;

    println!("{}", asteroid_coord.0 * 100 + asteroid_coord.1);

    // Wrong: 705 (off-by-one; 200 -> 199)

    Ok(())
}
