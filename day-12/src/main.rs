use itertools::Itertools;
use std::{
    ops::{AddAssign, SubAssign},
    str::FromStr,
};

type Error = Box<dyn std::error::Error>;
type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
struct Vector {
    x: i32,
    y: i32,
    z: i32,
}

impl Vector {
    fn magnitude(&self) -> i32 {
        self.x.abs() + self.y.abs() + self.z.abs()
    }
}

impl AddAssign for Vector {
    fn add_assign(&mut self, other: Vector) {
        self.x += other.x;
        self.y += other.y;
        self.z += other.z;
    }
}

impl SubAssign for Vector {
    fn sub_assign(&mut self, other: Vector) {
        self.x -= other.x;
        self.y -= other.y;
        self.z -= other.z;
    }
}

#[derive(Debug)]
struct System(Vec<Planet>);

impl System {
    fn step(&mut self, n: usize) {
        for _ in 0..n {
            self.step_once();
        }
    }

    fn step_once(&mut self) {
        let mut velocity_deltas = vec![Vector::default(); self.0.len()];

        let z = self.0.iter().enumerate();
        for ((ai, a), (bi, b)) in z.tuple_combinations() {
            let g = Planet::apply_gravity(a, b);

            velocity_deltas[ai] += g;
            velocity_deltas[bi] -= g;
        }

        for (planet, velocity_delta) in self.0.iter_mut().zip(velocity_deltas) {
            planet.velocity += velocity_delta;
            planet.position += planet.velocity;
        }
    }

    fn total_energy(&self) -> i32 {
        self.0.iter().map(Planet::total_energy).sum()
    }
}

impl FromStr for System {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        let p: Result<_, _> = s.trim().lines().map(str::parse).collect();
        Ok(System(p?))
    }
}

#[derive(Debug, Default, Copy, Clone, PartialEq, Eq)]
struct Planet {
    position: Vector,
    velocity: Vector,
}

impl Planet {
    fn apply_gravity(&self, other: &Self) -> Vector {
        fn z(a: i32, b: i32) -> i32 {
            use std::cmp::Ordering::*;

            match a.cmp(&b) {
                Greater => -1,
                Equal => 0,
                Less => 1,
            }
        }

        Vector {
            x: z(self.position.x, other.position.x),
            y: z(self.position.y, other.position.y),
            z: z(self.position.z, other.position.z),
        }
    }

    fn total_energy(&self) -> i32 {
        self.position.magnitude() * self.velocity.magnitude()
    }
}

impl FromStr for Planet {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        let coords = s.trim_start_matches("<").trim_end_matches(">").split(",");
        let mut numbers = coords.flat_map(|coord| {
            let mut values = coord.trim().split("=");
            values.nth(1).map(|n| n.parse())
        });

        let x = numbers.next().ok_or("Missing X coordinate")??;
        let y = numbers.next().ok_or("Missing Y coordinate")??;
        let z = numbers.next().ok_or("Missing Z coordinate")??;

        Ok(Planet {
            position: Vector { x, y, z },
            ..Default::default()
        })
    }
}

#[test]
fn basic_steps() -> Result<()> {
    let input = r#"
        <x=-1, y=0, z=2>
        <x=2, y=-10, z=-7>
        <x=4, y=-8, z=8>
        <x=3, y=5, z=-1>
    "#;
    let mut system: System = input.parse()?;
    assert_eq!(
        system.0[0],
        Planet {
            position: Vector { x: -1, y: 0, z: 2 },
            velocity: Vector { x: 0, y: 0, z: 0 },
        }
    );

    system.step_once();
    assert_eq!(
        system.0[0],
        Planet {
            position: Vector { x: 2, y: -1, z: 1 },
            velocity: Vector { x: 3, y: -1, z: -1 },
        }
    );
    // pos=<x= 3, y=-7, z=-4>, vel=<x= 1, y= 3, z= 3>
    // pos=<x= 1, y=-7, z= 5>, vel=<x=-3, y= 1, z=-3>
    // pos=<x= 2, y= 2, z= 0>, vel=<x=-1, y=-3, z= 1>

    system.step_once();
    assert_eq!(
        system.0[0],
        Planet {
            position: Vector { x: 5, y: -3, z: -1 },
            velocity: Vector { x: 3, y: -2, z: -2 },
        }
    );
    // pos=<x= 1, y=-2, z= 2>, vel=<x=-2, y= 5, z= 6>
    // pos=<x= 1, y=-4, z=-1>, vel=<x= 0, y= 3, z=-6>
    // pos=<x= 1, y=-4, z= 2>, vel=<x=-1, y=-6, z= 2>

    Ok(())
}

#[test]
fn total_energy() -> Result<()> {
    let input = r#"
        <x=-8, y=-10, z=0>
        <x=5, y=5, z=10>
        <x=2, y=-7, z=3>
        <x=9, y=-8, z=-3>
    "#;
    let mut system: System = input.parse()?;
    system.step(100);
    assert_eq!(system.total_energy(), 1940);
    Ok(())
}

const INPUT: &str = include_str!("input.txt");

fn main() -> Result<()> {
    let mut system: System = INPUT.parse()?;
    system.step(1000);
    println!("{}", system.total_energy());
    Ok(())
}
