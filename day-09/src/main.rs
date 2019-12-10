pub type Error = Box<dyn std::error::Error + Sync + Send + 'static>;
pub type Result<T, E = Error> = std::result::Result<T, E>;

const INPUT: &str = include_str!("input.txt");

fn main() -> Result<()> {
    let mut program = INPUT.split(",").flat_map(str::parse).collect();
    let output = intcode::execute(&mut program, Some(1))?;
    println!("{:?}", output);
    Ok(())
}
