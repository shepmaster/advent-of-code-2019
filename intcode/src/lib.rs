pub use crossbeam_channel::{unbounded as channel, Receiver, Sender};
use std::convert::TryInto;

pub type Byte = i32;
pub type Program = [Byte];
pub type ProgramCounter = usize;
pub type Output = Vec<Byte>;

pub type Error = Box<dyn std::error::Error + Sync + Send + 'static>;
pub type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, Copy, Clone, PartialEq)]
enum Parameter {
    Position(ProgramCounter),
    Immediate(Byte),
}

impl Parameter {
    fn read(&self, program: &Program) -> Byte {
        match *self {
            Parameter::Position(p) => program[p],
            Parameter::Immediate(i) => i,
        }
    }

    fn write(&self, program: &mut Program, value: Byte) {
        match *self {
            Parameter::Position(p) => program[p] = value,
            Parameter::Immediate(_) => panic!("Must not write to immediate parameter"),
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
    Halt,
}

impl Operation {
    fn decode(program: &mut Program, pc: ProgramCounter) -> Result<Self, Error> {
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
                let p = Self::decode_single_param(program, pc)?;
                Input(p)
            }
            04 => {
                let p = Self::decode_single_param(program, pc)?;
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
            99 => Halt,
            _ => Err(format!("Unknown opcode {}", opcode))?,
        })
    }

    fn decode_single_param(program: &Program, pc: ProgramCounter) -> Result<Parameter> {
        let raw_op = program[pc];
        let p1_mode = (raw_op % 1000 / 100) == 1;

        let args: [_; 1] = program[pc..][1..][..1].try_into()?;
        let [p] = args;

        Ok(if p1_mode {
            Parameter::Immediate(p)
        } else {
            Parameter::Position(p.try_into()?)
        })
    }

    fn decode_two_params(program: &Program, pc: ProgramCounter) -> Result<[Parameter; 2]> {
        let raw_op = program[pc];
        let p1_mode = (raw_op % 1000 / 100) == 1;
        let p2_mode = (raw_op % 10000 / 1000) == 1;

        let args: [_; 2] = program[pc..][1..][..2].try_into()?;
        let [a, b] = args;

        Ok([
            if p1_mode {
                Parameter::Immediate(a)
            } else {
                Parameter::Position(a.try_into()?)
            },
            if p2_mode {
                Parameter::Immediate(b)
            } else {
                Parameter::Position(b.try_into()?)
            },
        ])
    }

    fn decode_three_params(program: &Program, pc: ProgramCounter) -> Result<[Parameter; 3]> {
        let raw_op = program[pc];
        let p1_mode = (raw_op % 1000 / 100) == 1;
        let p2_mode = (raw_op % 10000 / 1000) == 1;
        let p3_mode = (raw_op % 100000 / 10000) == 1;

        let args: [_; 3] = program[pc..][1..][..3].try_into()?;
        let [l, r, o] = args;

        Ok([
            if p1_mode {
                Parameter::Immediate(l)
            } else {
                Parameter::Position(l.try_into()?)
            },
            if p2_mode {
                Parameter::Immediate(r)
            } else {
                Parameter::Position(r.try_into()?)
            },
            if p3_mode {
                Parameter::Immediate(o)
            } else {
                Parameter::Position(o.try_into()?)
            },
        ])
    }

    fn execute(
        &self,
        program: &mut Program,
        pc: &mut ProgramCounter,
        mut input: impl Iterator<Item = Byte>,
        mut output: impl OutputStream<Item = Byte>,
    ) -> Result<()> {
        use Operation::*;

        match self {
            Add(l, r, o) => {
                Self::binop(program, l, r, o, |l, r| l + r);
                *pc += self.width();
            }
            Multiply(l, r, o) => {
                Self::binop(program, l, r, o, |l, r| l * r);
                *pc += self.width();
            }
            Input(p) => {
                let v = input.next().ok_or("No more input is available")?;
                p.write(program, v);
                *pc += self.width();
            }
            Output(p) => {
                let v = p.read(program);
                output.push(v);
                *pc += self.width();
            }
            JumpIfTrue(c, l) => {
                if c.read(program) != 0 {
                    *pc = l.read(program).try_into()?;
                } else {
                    *pc += self.width();
                }
            }
            JumpIfFalse(c, l) => {
                if c.read(program) == 0 {
                    *pc = l.read(program).try_into()?;
                } else {
                    *pc += self.width();
                }
            }
            LessThan(l, r, o) => {
                let v = if l.read(program) < r.read(program) {
                    1
                } else {
                    0
                };
                o.write(program, v);
                *pc += self.width();
            }
            Equals(l, r, o) => {
                let v = if l.read(program) == r.read(program) {
                    1
                } else {
                    0
                };
                o.write(program, v);
                *pc += self.width();
            }
            Halt => { /* Do nothing */ }
        }

        Ok(())
    }

    fn binop(
        program: &mut Program,
        l: &Parameter,
        r: &Parameter,
        o: &Parameter,
        f: impl FnOnce(Byte, Byte) -> Byte,
    ) {
        let l = l.read(program);
        let r = r.read(program);
        let v = f(l, r);
        o.write(program, v);
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
            Halt => 1,
        }
    }
}

pub fn execute_with_output(
    program: &mut Program,
    input: impl IntoIterator<Item = Byte>,
    mut output: impl OutputStream<Item = Byte>,
) -> Result<()> {
    let mut pc = 0;
    let mut input = input.into_iter();

    loop {
        let op = Operation::decode(program, pc)?;

        if op == Operation::Halt {
            break;
        }

        op.execute(program, &mut pc, &mut input, &mut output)?;
    }

    Ok(())
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
        let mut state = [1, 0, 0, 0, 99];
        execute(&mut state, None)?;
        assert_eq!(state, [2, 0, 0, 0, 99]);

        let mut state = [2, 3, 0, 3, 99];
        execute(&mut state, None)?;
        assert_eq!(state, [2, 3, 0, 6, 99]);

        let mut state = [2, 4, 4, 5, 99, 0];
        execute(&mut state, None)?;
        assert_eq!(state, [2, 4, 4, 5, 99, 9801]);

        let mut state = [1, 1, 1, 4, 99, 5, 6, 0, 99];
        execute(&mut state, None)?;
        assert_eq!(state, [30, 1, 1, 4, 2, 5, 6, 0, 99]);

        Ok(())
    }

    #[test]
    fn specifications_day_05() -> Result<()> {
        let mut program = [1002, 4, 3, 4, 33];
        execute(&mut program, None)?;
        assert_eq!(program[4], 99);

        let mut program = [3, 0, 4, 0, 99];
        let output = execute(&mut program, Some(42))?;
        assert_eq!(output, [42]);

        Ok(())
    }

    #[test]
    fn compare_instructions() -> Result<()> {
        let mut program = [3, 9, 8, 9, 10, 9, 4, 9, 99, -1, 8];
        let output = execute(&mut program, Some(8))?;
        assert_eq!(output, [1]);

        let mut program = [3, 9, 7, 9, 10, 9, 4, 9, 99, -1, 8];
        let output = execute(&mut program, Some(8))?;
        assert_eq!(output, [0]);

        let mut program = [3, 3, 1108, -1, 8, 3, 4, 3, 99];
        let output = execute(&mut program, Some(7))?;
        assert_eq!(output, [0]);

        let mut program = [3, 3, 1107, -1, 8, 3, 4, 3, 99];
        let output = execute(&mut program, Some(7))?;
        assert_eq!(output, [1]);

        Ok(())
    }

    #[test]
    fn jump_instructions() -> Result<()> {
        let mut program = [3, 12, 6, 12, 15, 1, 13, 14, 13, 4, 13, 99, -1, 0, 1, 9];
        let output = execute(&mut program, Some(0))?;
        assert_eq!(output, [0]);

        let mut program = [3, 3, 1105, -1, 9, 1101, 0, 0, 12, 4, 12, 99, 1];
        let output = execute(&mut program, Some(100))?;
        assert_eq!(output, [1]);

        Ok(())
    }
}
