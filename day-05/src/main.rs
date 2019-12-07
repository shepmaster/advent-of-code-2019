use std::convert::TryInto;

type Byte = i32;
type Program = [Byte];
type ProgramCounter = usize;
type Output = Vec<Byte>;
type Error = Box<dyn std::error::Error>;
type Result<T, E = Error> = std::result::Result<T, E>;

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
    pub fn decode(program: &mut Program, pc: ProgramCounter) -> Result<Self, Error> {
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

    pub fn execute(
        &self,
        program: &mut Program,
        pc: &mut ProgramCounter,
        mut input: impl Iterator<Item = Byte>,
        output: &mut Output,
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

    pub fn width(&self) -> ProgramCounter {
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

fn execute(program: &mut Program, input: impl IntoIterator<Item = Byte>) -> Result<Output> {
    let mut pc = 0;
    let mut input = input.into_iter();
    let mut output = Output::default();

    loop {
        let op = Operation::decode(program, pc)?;

        if op == Operation::Halt {
            break;
        }

        op.execute(program, &mut pc, &mut input, &mut output)?;
    }

    Ok(output)
}

#[test]
fn specifications() -> Result<()> {
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

const INPUT: &str = include_str!("input.txt");

fn main() -> Result<()> {
    let mut program = INPUT
        .trim()
        .split(",")
        .map(str::parse)
        .collect::<Result<Vec<_>, _>>()?;

    let output = execute(&mut program, Some(5))?;
    println!("{:?}", output);

    Ok(())
}
