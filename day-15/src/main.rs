use std::collections::BTreeMap;

type Error = Box<dyn std::error::Error + Send + Sync + 'static>;
type Result<T, E = Error> = std::result::Result<T, E>;

type Coord = (i32, i32);
type Tile = intcode::Byte;
type Map = BTreeMap<Coord, Tile>;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum Direction {
    North = 1,
    South = 2,
    West = 3,
    East = 4,
}

impl Direction {
    fn apply_to(self, coord: Coord) -> Coord {
        use Direction::*;

        let (x, y) = coord;
        match self {
            North => (x, y + 1),
            South => (x, y - 1),
            West => (x - 1, y),
            East => (x + 1, y),
        }
    }

    fn turn_left(self) -> Self {
        use Direction::*;

        match self {
            North => West,
            South => East,
            West => South,
            East => North,
        }
    }

    fn turn_right(self) -> Self {
        use Direction::*;

        match self {
            North => East,
            South => West,
            West => North,
            East => South,
        }
    }
}

const WALL: Tile = 0;
const MOVE: Tile = 1;
const OXYGEN: Tile = 2;

fn print_map(map: &Map, bot: Coord) {
    if let util::BoundsCollect(Some(bounds)) = map.keys().copied().collect() {
        for y in (bounds.min_y..=bounds.max_y).rev() {
            for x in bounds.min_x..=bounds.max_x {
                let c = if (x, y) == bot {
                    'x'
                } else {
                    match map.get(&(x, y)) {
                        Some(&WALL) => '█',
                        Some(&MOVE) => ' ',
                        Some(&OXYGEN) => 'o',
                        _ => '⋅',
                    }
                };
                print!("{}", c);
            }
            println!();
        }
    }
}

#[derive(Debug)]
enum SearchMode {
    FindInitialWall,
    Navigate(Direction),
    FollowWall(Direction, Direction),
}

impl SearchMode {
    fn next_direction_and_position(&mut self, current: Coord, map: &Map) -> (Direction, Coord) {
        use Direction::*;
        use SearchMode::*;

        match self {
            FindInitialWall => {
                let north = North.apply_to(current);
                match map.get(&north).copied() {
                    Some(WALL) => {
                        *self = Navigate(East);
                        self.next_direction_and_position(current, map)
                    }
                    _ => (North, north),
                }
            }

            Navigate(forward_dir) => {
                let forward_coord = forward_dir.apply_to(current);
                let forward_tile = map.get(&forward_coord).copied();

                let left_dir = forward_dir.turn_left();
                let left_coord = left_dir.apply_to(current);
                let left_tile = map.get(&left_coord).copied();

                let right_dir = forward_dir.turn_right();

                if let None | Some(MOVE) | Some(OXYGEN) = left_tile {
                    *self = FollowWall(left_dir, *forward_dir);
                    return (left_dir, left_coord);
                }

                if let Some(WALL) = forward_tile {
                    *forward_dir = right_dir;
                    return self.next_direction_and_position(current, map);
                }

                (*forward_dir, forward_coord)
            }

            FollowWall(maybe_dir, original_dir) => {
                let maybe_coord = maybe_dir.apply_to(current);
                let maybe_tile = map.get(&maybe_coord).copied();

                if let Some(WALL) = maybe_tile {
                    *self = Navigate(*original_dir);
                } else {
                    *self = Navigate(*maybe_dir);
                }

                self.next_direction_and_position(current, map)
            }
        }
    }
}

fn map_out_area(computer: &mut intcode::Computer) -> Map {
    computer.execute_side_by_side(|tx, rx| {
        let mut map = BTreeMap::new();
        let mut position = (0, 0);
        let mut search_mode = SearchMode::FindInitialWall;

        let mut rx = rx.into_iter();

        loop {
            if position == (0, 0) && neighbors(position).all(|n| map.get(&n).is_some()) {
                // We've returned to the start and visited everywhere
                print_map(&map, (0, 0));

                // The program never seems to halt, so we do work here
                // and trigger our own panic to print out the data.
                let path = calculate_path(&map);
                println!("Found oxygen {} steps away", path.len() - 1);

                let time = calculate_dispersion(map);
                println!("Oxygen takes {} minutes to fill the area", time);

                panic!("Program doesn't halt");
            }

            let (direction, next_position) =
                search_mode.next_direction_and_position(position, &map);

            tx.send(direction as intcode::Byte)
                .expect("Computer has shut down");

            let next_contents = rx.next().expect("No response from computer");

            map.insert(next_position, next_contents);
            match next_contents {
                WALL => { /* Do nothing */ }
                MOVE => position = next_position,
                OXYGEN => position = next_position,
                _ => panic!("Unknown tile"),
            }
        }
    })
}

fn calculate_path(map: &Map) -> Vec<Coord> {
    if let util::BoundsCollect(Some(bounds)) = map.keys().copied().collect() {
        let mut graph = petgraph::graphmap::UnGraphMap::new();
        let start = graph.add_node((0, 0));

        let oxygen_coord = map
            .iter()
            .find(|&(_, &t)| t == OXYGEN)
            .map(|(c, _)| c)
            .copied()
            .expect("Oxygen is not in the map");

        for y in bounds.min_y..=bounds.max_y {
            for x in bounds.min_x..=bounds.max_x {
                let coord = (x, y);
                if let Some(MOVE) | Some(OXYGEN) = map.get(&coord).copied() {
                    for neighbor in neighbors(coord) {
                        if let Some(MOVE) | Some(OXYGEN) = map.get(&neighbor).copied() {
                            graph.add_edge(coord, neighbor, 1);
                        }
                    }
                }
            }
        }

        let (_length, path) = petgraph::algo::astar(
            &graph,
            start,
            /* is_goal: */ |node| node == oxygen_coord,
            /* edge_cost: */ |_| 1,
            /* estimate_cost: */ |_| 1,
        )
        .expect("No path found");
        return path;
    }

    vec![]
}

fn calculate_dispersion(mut map: Map) -> usize {
    let mut minutes = 0;

    while map.values().any(|&t| t == MOVE) {
        let expansion_coords = map
            .iter()
            .filter(|&(_, &t)| t == OXYGEN)
            .flat_map(|(&c, _)| neighbors(c))
            .filter(|c| map.get(c).copied() == Some(MOVE));

        let mut next_map = map.clone();
        for expansion_coord in expansion_coords {
            next_map.insert(expansion_coord, OXYGEN);
        }
        map = next_map;

        minutes += 1;
    }

    minutes
}

fn neighbors(coord: Coord) -> impl Iterator<Item = Coord> {
    let (x, y) = coord;

    vec![(x - 1, y), (x + 1, y), (x, y - 1), (x, y + 1)].into_iter()
}

const INPUT: &str = include_str!("input.txt");

fn main() -> Result<()> {
    let mut computer: intcode::Computer = INPUT.parse()?;
    map_out_area(&mut computer);
    Ok(())

    // WRONG: 39
    // WRONG: 38
    // Had a copy-paste typo in the graph
    // WRONG: 271
    // Off-by-one: That's the number of nodes, not steps
}
