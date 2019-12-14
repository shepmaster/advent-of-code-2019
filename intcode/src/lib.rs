pub use crossbeam_channel::{unbounded as channel, Receiver, Sender};
use crossbeam_utils::thread;
use itertools::Itertools;
use std::{convert::{TryFrom, TryInto}, str::FromStr};

pub type Byte = i128;
pub type Program = Vec<Byte>;
pub type ProgramCounter = usize;
pub type ProgramCounterOffset = isize;
pub type Output = Vec<Byte>;

pub type Error = Box<dyn std::error::Error + Sync + Send + 'static>;
pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, Copy, Clone, PartialEq)]
enum Parameter {
    Position(ProgramCounter),
    Immediate(Byte),
    Relative(ProgramCounterOffset),
}

impl Parameter {
    fn from_mode_and_value(mode: Byte, value: Byte) -> Result<Self> {
        match mode {
            0 => Ok(Parameter::Position(value.try_into()?)),
            1 => Ok(Parameter::Immediate(value)),
            2 => Ok(Parameter::Relative(value.try_into()?)),
            _ => Err(format!("Unknown mode {}", mode))?,
        }
    }

    fn read(&self, program: &Program, relative_base: ProgramCounter) -> Byte {
        match *self {
            Parameter::Position(p) => program.get(p).copied().unwrap_or(0),
            Parameter::Immediate(i) => i,
            Parameter::Relative(r) => {
                let b = isize::try_from(relative_base).expect("Cannot convert relative base");
                let a = usize::try_from(b + r).expect("Cannot convert absolute address");
                program.get(a).copied().unwrap_or(0)
            }
        }
    }

    fn write(&self, program: &mut Program, relative_base: ProgramCounter, value: Byte) {
        match *self {
            Parameter::Position(p) => {
                if program.len() <= p {
                    program.resize(p + 1, 0);
                }
                program[p] = value
            }
            Parameter::Immediate(_) => panic!("Must not write to immediate parameter"),
            Parameter::Relative(r) => {
                let b = isize::try_from(relative_base).expect("Cannot convert relative base");
                let a = usize::try_from(b + r).expect("Cannot convert absolute address");
                if program.len() <= a {
                    program.resize(a + 1, 0);
                }
                program[a] = value;
            }
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
enum Operation {
    Add(Parameter, Parameter, Parameter),
    Multiply(Parameter, Parameter, Parameter),
    Input(Parameter),
    Output(Parameter),
    JumpIfTrue(Parameter, Parameter),
    JumpIfFalse(Parameter, Parameter),
    LessThan(Parameter, Parameter, Parameter),
    Equals(Parameter, Parameter, Parameter),
    AdjustRelativeBase(Parameter),
    Halt,
}

impl Operation {
    fn decode(program: &Program, pc: ProgramCounter) -> Result<Self, Error> {
        use Operation::*;

        let opcode = program[pc] % 100;

        Ok(match opcode {
            01 => {
                let [l, r, o] = Self::decode_three_params(program, pc)?;
                Add(l, r, o)
            }
            02 => {
                let [l, r, o] = Self::decode_three_params(program, pc)?;
                Multiply(l, r, o)
            }
            03 => {
                let [p] = Self::decode_single_param(program, pc)?;
                Input(p)
            }
            04 => {
                let [p] = Self::decode_single_param(program, pc)?;
                Output(p)
            }
            05 => {
                let [c, l] = Self::decode_two_params(program, pc)?;
                JumpIfTrue(c, l)
            }
            06 => {
                let [c, l] = Self::decode_two_params(program, pc)?;
                JumpIfFalse(c, l)
            }
            07 => {
                let [l, r, o] = Self::decode_three_params(program, pc)?;
                LessThan(l, r, o)
            }
            08 => {
                let [l, r, o] = Self::decode_three_params(program, pc)?;
                Equals(l, r, o)
            }
            09 => {
                let [p] = Self::decode_single_param(program, pc)?;
                AdjustRelativeBase(p)
            }
            99 => Halt,
            _ => Err(format!("Unknown opcode {}", opcode))?,
        })
    }

    fn decode_single_param(program: &Program, pc: ProgramCounter) -> Result<[Parameter; 1]> {
        let (a,) = Self::params(program, pc)
            .tuples()
            .next()
            .ok_or("Not enough arguments")?;

        Ok([a?])
    }

    fn decode_two_params(program: &Program, pc: ProgramCounter) -> Result<[Parameter; 2]> {
        let (a, b) = Self::params(program, pc)
            .tuples()
            .next()
            .ok_or("Not enough arguments")?;

        Ok([a?, b?])
    }

    fn decode_three_params(program: &Program, pc: ProgramCounter) -> Result<[Parameter; 3]> {
        let (a, b, c) = Self::params(program, pc)
            .tuples()
            .next()
            .ok_or("Not enough arguments")?;

        Ok([a?, b?, c?])
    }

    fn params(
        program: &Program,
        pc: ProgramCounter,
    ) -> impl Iterator<Item = Result<Parameter>> + '_ {
        let (op, args) = program[pc..].split_at(1);

        Self::modes(op[0])
            .zip(args)
            .map(|(m, &v)| Parameter::from_mode_and_value(m, v))
    }

    fn modes(raw_op: Byte) -> impl Iterator<Item = Byte> {
        let mut a = 100;
        (0..).map(move |_| {
            let v = raw_op % (a * 10) / a;
            a *= 10;
            v
        })
    }

    fn execute(
        &self,
        program: &mut Program,
        pc: &mut ProgramCounter,
        relative_base: &mut ProgramCounter,
        mut input: impl Iterator<Item = Byte>,
        mut output: impl OutputStream<Item = Byte>,
    ) -> Result<()> {
        use Operation::*;

        match self {
            Add(l, r, o) => {
                Self::binop(program, *relative_base, l, r, o, |l, r| l + r);
                *pc += self.width();
            }
            Multiply(l, r, o) => {
                Self::binop(program, *relative_base, l, r, o, |l, r| l * r);
                *pc += self.width();
            }
            Input(p) => {
                let v = input.next().ok_or("No more input is available")?;
                p.write(program, *relative_base, v);
                *pc += self.width();
            }
            Output(p) => {
                let v = p.read(program, *relative_base);
                output.push(v);
                *pc += self.width();
            }
            JumpIfTrue(c, l) => {
                if c.read(program, *relative_base) != 0 {
                    *pc = l.read(program, *relative_base).try_into()?;
                } else {
                    *pc += self.width();
                }
            }
            JumpIfFalse(c, l) => {
                if c.read(program, *relative_base) == 0 {
                    *pc = l.read(program, *relative_base).try_into()?;
                } else {
                    *pc += self.width();
                }
            }
            LessThan(l, r, o) => {
                let v = if l.read(program, *relative_base) < r.read(program, *relative_base) {
                    1
                } else {
                    0
                };
                o.write(program, *relative_base, v);
                *pc += self.width();
            }
            Equals(l, r, o) => {
                let v = if l.read(program, *relative_base) == r.read(program, *relative_base) {
                    1
                } else {
                    0
                };
                o.write(program, *relative_base, v);
                *pc += self.width();
            }
            AdjustRelativeBase(p) => {
                let r = p.read(program, *relative_base);
                let r = isize::try_from(r).expect("Cannot convert relative offset");
                let b = isize::try_from(*relative_base).expect("Cannot convert relative base");
                let a = usize::try_from(b + r).expect("Cannot convert absolute address");
                *relative_base = a;
                *pc += self.width();
            }
            Halt => *pc += self.width(),
        }

        Ok(())
    }

    fn binop(
        program: &mut Program,
        relative_base: ProgramCounter,
        l: &Parameter,
        r: &Parameter,
        o: &Parameter,
        f: impl FnOnce(Byte, Byte) -> Byte,
    ) {
        let l = l.read(program, relative_base);
        let r = r.read(program, relative_base);
        let v = f(l, r);
        o.write(program, relative_base, v);
    }

    fn width(&self) -> ProgramCounter {
        use Operation::*;

        match self {
            Add(..) => 4,
            Multiply(..) => 4,
            Input(..) => 2,
            Output(..) => 2,
            JumpIfTrue(..) => 3,
            JumpIfFalse(..) => 3,
            LessThan(..) => 4,
            Equals(..) => 4,
            AdjustRelativeBase(..) => 2,
            Halt => 1,
        }
    }
}

pub fn parse_program(s: &str) -> Program {
    s.trim().split(",").flat_map(str::parse).collect()
}

#[derive(Debug)]
pub struct Computer {
    pub program: Program,
    pc: usize,
    relative_base: usize,
}

impl FromStr for Computer {
    type Err = Error;

    fn from_str(s: &str) -> Result<Computer> {
        let program: Result<_, _> = s.trim().split(",").map(str::parse).collect();
        Ok(Self::new(program?))
    }
}

impl Computer {
    pub fn new(program: Program) -> Self {
        Self {
            program,
            pc: 0,
            relative_base: 0,
        }
    }

    pub fn execute(
        &mut self,
        input: impl IntoIterator<Item = Byte>,
        mut output: impl OutputStream<Item = Byte>,
    ) -> Result<()> {
        let mut input = input.into_iter();

        loop {
            let op = Operation::decode(&self.program, self.pc)?;

            op.execute(
                &mut self.program,
                &mut self.pc,
                &mut self.relative_base,
                &mut input,
                &mut output,
            )?;

            if op == Operation::Halt {
                break;
            }
        }

        Ok(())
    }

    pub fn execute_side_by_side<F, T>(&mut self, f: F) -> T
    where
        T: Send + Sync,
        F: FnOnce(Sender<Byte>, Receiver<Byte>) -> T,
        F: Send + Sync,
    {
        let (tx, rx) = channel();
        let (tx2, rx2) = channel();

        thread::scope(|s| {
            let side_program = s.spawn(move |_| f(tx2, rx));

            let computer = s.spawn(move |_| {
                eprintln!("Computer running");
                let v = self.execute(rx2, tx);
                eprintln!("Computer exiting");
                v
            });

            computer
                .join()
                .expect("Computer panicked")
                .expect("Execution failed");
            let side_result = side_program.join().expect("Side program panicked");

            side_result
        })
        .expect("Unable to run side-by-side program")
    }
}

pub fn execute_with_output(
    program: &mut Program,
    input: impl IntoIterator<Item = Byte>,
    output: impl OutputStream<Item = Byte>,
) -> Result<()> {
    let inner_program = std::mem::replace(program, Program::new());
    let mut computer = Computer::new(inner_program);
    let r = computer.execute(input, output);
    std::mem::swap(program, &mut computer.program);
    r
}

pub fn execute_side_by_side<F, T>(program: &mut Program, f: F) -> T
where
    T: Send + Sync,
    F: FnOnce(Sender<Byte>, Receiver<Byte>) -> T,
    F: Send + Sync,
{
    let inner_program = std::mem::replace(program, Program::new());
    let mut computer = Computer::new(inner_program);
    let r = computer.execute_side_by_side(f);
    std::mem::swap(program, &mut computer.program);
    r
}

pub fn execute(program: &mut Program, input: impl IntoIterator<Item = Byte>) -> Result<Output> {
    let mut output = Output::default();
    execute_with_output(program, input, &mut output).map(|()| output)
}

pub trait OutputStream {
    type Item;
    fn push(&mut self, val: Self::Item);
}

impl<O> OutputStream for &'_ mut O
where
    O: OutputStream,
{
    type Item = O::Item;

    fn push(&mut self, val: Self::Item) {
        (**self).push(val);
    }
}

impl<T> OutputStream for Vec<T> {
    type Item = T;

    fn push(&mut self, val: Self::Item) {
        Vec::push(self, val);
    }
}

impl<T> OutputStream for Sender<T> {
    type Item = T;

    fn push(&mut self, val: Self::Item) {
        self.send(val).expect("Unable to output to channel");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn specifications_day_02() -> Result<()> {
        let mut state = vec![1, 0, 0, 0, 99];
        execute(&mut state, None)?;
        assert_eq!(state, [2, 0, 0, 0, 99]);

        let mut state = vec![2, 3, 0, 3, 99];
        execute(&mut state, None)?;
        assert_eq!(state, [2, 3, 0, 6, 99]);

        let mut state = vec![2, 4, 4, 5, 99, 0];
        execute(&mut state, None)?;
        assert_eq!(state, [2, 4, 4, 5, 99, 9801]);

        let mut state = vec![1, 1, 1, 4, 99, 5, 6, 0, 99];
        execute(&mut state, None)?;
        assert_eq!(state, [30, 1, 1, 4, 2, 5, 6, 0, 99]);

        Ok(())
    }

    #[test]
    fn specifications_day_05() -> Result<()> {
        let mut program = vec![1002, 4, 3, 4, 33];
        execute(&mut program, None)?;
        assert_eq!(program[4], 99);

        let mut program = vec![3, 0, 4, 0, 99];
        let output = execute(&mut program, Some(42))?;
        assert_eq!(output, [42]);

        Ok(())
    }

    #[test]
    fn compare_instructions() -> Result<()> {
        let mut program = vec![3, 9, 8, 9, 10, 9, 4, 9, 99, -1, 8];
        let output = execute(&mut program, Some(8))?;
        assert_eq!(output, [1]);

        let mut program = vec![3, 9, 7, 9, 10, 9, 4, 9, 99, -1, 8];
        let output = execute(&mut program, Some(8))?;
        assert_eq!(output, [0]);

        let mut program = vec![3, 3, 1108, -1, 8, 3, 4, 3, 99];
        let output = execute(&mut program, Some(7))?;
        assert_eq!(output, [0]);

        let mut program = vec![3, 3, 1107, -1, 8, 3, 4, 3, 99];
        let output = execute(&mut program, Some(7))?;
        assert_eq!(output, [1]);

        Ok(())
    }

    #[test]
    fn jump_instructions() -> Result<()> {
        let mut program = vec![3, 12, 6, 12, 15, 1, 13, 14, 13, 4, 13, 99, -1, 0, 1, 9];
        let output = execute(&mut program, Some(0))?;
        assert_eq!(output, [0]);

        let mut program = vec![3, 3, 1105, -1, 9, 1101, 0, 0, 12, 4, 12, 99, 1];
        let output = execute(&mut program, Some(100))?;
        assert_eq!(output, [1]);

        Ok(())
    }

    #[test]
    fn quine() -> Result<()> {
        let original = [
            109, 1, 204, -1, 1001, 100, 1, 100, 1008, 100, 16, 101, 1006, 101, 0, 99,
        ];
        let mut program = original.to_vec();
        let output = execute(&mut program, None)?;
        assert_eq!(output, original);

        Ok(())
    }

    #[test]
    fn sixteen_digit() -> Result<()> {
        let mut program = vec![1102, 34915192, 34915192, 7, 4, 7, 99, 0];
        let output = execute(&mut program, None)?;
        assert_eq!(output.len(), 1);
        assert_eq!(output[0].to_string().len(), 16);
        Ok(())
    }

    #[test]
    fn big_number() -> Result<()> {
        let mut program = vec![104, 1125899906842624, 99];
        let output = execute(&mut program, None)?;
        assert_eq!(output.len(), 1);
        assert_eq!(output[0], 1125899906842624);
        Ok(())
    }
}
