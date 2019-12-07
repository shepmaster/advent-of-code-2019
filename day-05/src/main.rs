type Error = Box<dyn std::error::Error>;
type Result<T, E = Error> = std::result::Result<T, E>;

const INPUT: &str = include_str!("input.txt");

fn main() -> Result<()> {
    let mut program = INPUT
        .trim()
        .split(",")
        .map(str::parse)
        .collect::<Result<Vec<_>, _>>()?;

    let output = intcode::execute(&mut program, Some(5))?;
    println!("{:?}", output);

    Ok(())
}
