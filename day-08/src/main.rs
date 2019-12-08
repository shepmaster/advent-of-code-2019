use std::collections::BTreeMap;

pub type Error = Box<dyn std::error::Error + Send + Sync + 'static>;
pub type Result<T, E = Error> = std::result::Result<T, E>;

fn checksum(data: &str, width: usize, height: usize) -> Result<usize> {
    let mut digits = data.split("").flat_map(str::parse).peekable();

    let mut layers = Vec::new();
    while digits.peek().is_some() {
        layers.push(digits.by_ref().take(width * height).collect::<Vec<usize>>());
    }

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

#[test]
fn specifications() -> Result<()> {
    let data = "123456789012";
    assert_eq!(checksum(data, 3, 2)?, 1);
    Ok(())
}

const INPUT: &str = include_str!("input.txt");

fn main() {
    println!("{:?}", checksum(INPUT, 25, 6));
}
