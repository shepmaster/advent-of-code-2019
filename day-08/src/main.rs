use itertools::Itertools;
use std::collections::BTreeMap;

pub type Error = Box<dyn std::error::Error + Send + Sync + 'static>;
pub type Result<T, E = Error> = std::result::Result<T, E>;

fn layers(data: &str, width: usize, height: usize) -> Result<Vec<Vec<usize>>> {
    let mut digits = data.split("").flat_map(str::parse).peekable();

    let mut layers = Vec::new();
    while digits.peek().is_some() {
        layers.push(digits.by_ref().take(width * height).collect::<Vec<usize>>());
    }
    Ok(layers)
}

fn checksum(data: &str, width: usize, height: usize) -> Result<usize> {
    let layers = layers(data, width, height)?;

    let counts: Vec<_> = layers
        .iter()
        .map(|l| {
            let mut counts = BTreeMap::new();
            for &d in l {
                *counts.entry(d).or_insert(0usize) += 1;
            }
            counts
        })
        .collect();

    let min_layer = counts
        .iter()
        .min_by_key(|c| c.get(&0))
        .ok_or("No layers with zero")?;

    let ones = min_layer.get(&1).ok_or("No ones in layer")?;
    let twos = min_layer.get(&2).ok_or("No twos in layer")?;

    Ok(ones * twos)
}

fn composite(data: &str, width: usize, height: usize) -> Result<String> {
    let layers = layers(data, width, height)?;
    let composed_layer = layers
        .into_iter()
        .fold1(|comp, layer| {
            comp.into_iter()
                .zip(layer)
                .map(|(c, l)| match c {
                    2 => l,
                    _ => c,
                })
                .collect()
        })
        .ok_or("Did not have any layers")?;

    let mut layer_string = String::new();
    for line in composed_layer.chunks(width) {
        use std::fmt::Write;
        for n in line {
            write!(&mut layer_string, "{}", n)?;
        }
        writeln!(&mut layer_string)?;
    }
    Ok(layer_string)
}

#[test]
fn specifications_checksum() -> Result<()> {
    let data = "123456789012";
    assert_eq!(checksum(data, 3, 2)?, 1);
    Ok(())
}

#[test]
fn specifications_composite() -> Result<()> {
    let data = "0222112222120000";
    assert_eq!(composite(data, 2, 2)?, "01\n10\n");
    Ok(())
}

const INPUT: &str = include_str!("input.txt");

fn main() -> Result<()> {
    println!("{}", checksum(INPUT, 25, 6)?);

    let composite = composite(INPUT, 25, 6)?;
    for mut c in composite.chars() {
        if c == '0' {
            c = ' '
        }
        print!("{}", c);
    }

    Ok(())
}
