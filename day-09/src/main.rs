pub type Error = Box<dyn std::error::Error + Sync + Send + 'static>;
pub type Result<T, E = Error> = std::result::Result<T, E>;

const INPUT: &str = include_str!("input.txt");

fn main() -> Result<()> {
    let mut program = intcode::parse_program(INPUT);

    let output = intcode::execute(&mut program.clone(), Some(1))?;
    println!("{:?}", output);

    let output = intcode::execute(&mut program, Some(2))?;
    println!("{:?}", output);

    Ok(())
}
