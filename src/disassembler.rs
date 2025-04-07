use instruction::{
    Alu, COrImmediate, Direction, Instruction, JumpTest, LoadIndirect, LoadType, Register,
    Register16, Register16Alt, RegisterOrImmediate, Rot,
};
use nom::{
    IResult, Parser, bits,
    error::Error,
    number::{le_i8, le_u8, le_u16},
};
use num_traits::FromPrimitive as _;

use crate::disassembler::instruction::HLOrImmediate;

pub mod instruction;

#[allow(
    clippy::many_single_char_names,
    clippy::too_many_lines,
    clippy::cognitive_complexity
)]
pub fn parse_instruction(i: &[u8]) -> IResult<&[u8], Instruction> {
    // based on <https://gb-archive.github.io/salvage/decoding_gbz80_opcodes/Decoding%20Gamboy%20Z80%20Opcodes.html>
    let (i, byte) = nom::bytes::complete::take(1usize).parse(i)?;
    let byte = byte[0];
    let x = (byte & 0b1100_0000) >> 6;
    let y = (byte & 0b0011_1000) >> 3;
    let z = byte & 0b0000_0111;
    assert!(x < 4);
    assert!(y < 8);
    assert!(z < 8);
    let p = y >> 1;
    let q = y % 2;

    let nyi = || todo!("instruction parse for X:{x} Z:{z} Y:{y} P:{p} Q:{q}");
    let unreachable = || {
        format!(
            "impossible state! X:{x} Z:{z} Y:{y} P:{p} Q:{q}\ndid you increment pc incorrectly?"
        )
    };

    Ok(match x {
        0 => match z {
            0 => match y {
                0 => (i, Instruction::Nop),
                1 => {
                    let (i, address) = le_u16().parse(i)?;
                    (
                        i,
                        Instruction::Ld(LoadType::FromSp(HLOrImmediate::Immediate(address), 0)),
                    )
                }
                3 => {
                    let (i, relative) = le_i8().parse(i)?;
                    (i, Instruction::JR(JumpTest::Always, relative))
                }
                4..8 => {
                    let condition = JumpTest::from_u8(y - 4).unwrap();
                    let (i, relative) = le_i8().parse(i)?;
                    (i, Instruction::JR(condition, relative))
                }
                _ => nyi(),
            },
            1 => match q {
                0 => {
                    let (i, target) = le_u16().parse(i)?;
                    let reg = Register16::from_u8(p).unwrap();

                    (
                        i,
                        Instruction::Ld(LoadType::Word(reg, HLOrImmediate::Immediate(target))),
                    )
                }
                1 => {
                    let reg = Register16::from_u8(p).unwrap();
                    (i, Instruction::AddHl(reg))
                }
                _ => unreachable!("{}", unreachable()),
            },
            2 => {
                let direction = match q {
                    0 => Direction::FromA,
                    1 => Direction::IntoA,
                    _ => unreachable!("{}", unreachable()),
                };
                let indirect_type = match p {
                    0 => LoadIndirect::BC,
                    1 => LoadIndirect::DE,
                    2 => LoadIndirect::HLInc,
                    3 => LoadIndirect::HLDec,
                    _ => unreachable!("{}", unreachable()),
                };
                (
                    i,
                    Instruction::Ld(LoadType::Indirect(indirect_type, direction)),
                )
            }
            3 => {
                let reg = Register16::from_u8(p).unwrap();
                match q {
                    0 => (i, Instruction::Inc16(reg)),
                    1 => (i, Instruction::Dec16(reg)),
                    _ => unreachable!("{}", unreachable()),
                }
            }
            4 => {
                let reg = Register::from_u8(y).unwrap();
                (i, Instruction::Inc(reg))
            }
            5 => {
                let reg = Register::from_u8(y).unwrap();
                (i, Instruction::Dec(reg))
            }
            6 => {
                let reg = Register::from_u8(y).unwrap();
                let (i, value) = le_u8().parse(i)?;
                (
                    i,
                    Instruction::Ld(LoadType::Byte(reg, RegisterOrImmediate::Immediate(value))),
                )
            }
            7 => match y {
                0 => (i, Instruction::Rlca),
                1 => (i, Instruction::Rrca),
                2 => (i, Instruction::Rla),
                3 => (i, Instruction::Rra),
                4 => (i, Instruction::Daa),
                5 => (i, Instruction::Cpl),
                6 => (i, Instruction::Scf),
                7 => (i, Instruction::Ccf),
                _ => unreachable!("{}", unreachable()),
            },
            _ => unreachable!("{}", unreachable()),
        },
        1 => match z {
            6 if y == 6 => (i, Instruction::Halt),
            0..8 => {
                let target = Register::from_u8(y).unwrap();
                let source = Register::from_u8(z).unwrap();

                (
                    i,
                    Instruction::Ld(LoadType::Byte(
                        target,
                        RegisterOrImmediate::Register(source),
                    )),
                )
            }
            _ => unreachable!("{}", unreachable()),
        },
        2 => {
            let reg = Register::from_u8(z).unwrap();
            let alu = Alu::from_u8(y).unwrap();
            (
                i,
                Instruction::Arithmetic(alu, RegisterOrImmediate::Register(reg)),
            )
        }
        3 => match z {
            0 => match y {
                0..4 => {
                    let condition = JumpTest::from_u8(y).unwrap();
                    (i, Instruction::Ret(condition))
                }
                4 | 6 => {
                    let (i, value) = le_u8().parse(i)?;
                    let direction = match y {
                        4 => Direction::FromA,
                        6 => Direction::IntoA,
                        _ => unreachable!(),
                    };
                    (
                        i,
                        Instruction::Ld(LoadType::LastByteAddress(
                            COrImmediate::Immediate(value),
                            direction,
                        )),
                    )
                }
                5 => {
                    let (i, offset) = le_i8().parse(i)?;
                    (i, Instruction::AddSp(offset))
                }
                7 => {
                    let (i, offset) = le_i8().parse(i)?;
                    (
                        i,
                        Instruction::Ld(LoadType::FromSp(HLOrImmediate::HL, offset)),
                    )
                }
                _ => nyi(),
            },
            1 => match q {
                0 => {
                    let reg = Register16Alt::from_u8(p).unwrap();
                    (i, Instruction::Pop(reg))
                }
                1 => match p {
                    0 => (i, Instruction::Ret(JumpTest::Always)),
                    1 => (i, Instruction::Reti),

                    2 => (i, Instruction::JP(JumpTest::Always, HLOrImmediate::HL)),
                    3 => (
                        i,
                        Instruction::Ld(LoadType::Word(Register16::SP, HLOrImmediate::HL)),
                    ),
                    _ => unreachable!("{}", unreachable()),
                },
                _ => unreachable!("{}", unreachable()),
            },
            2 => match y {
                0..4 => {
                    let (i, address) = le_u16().parse(i)?;
                    let condition = JumpTest::from_u8(y).unwrap();
                    (
                        i,
                        Instruction::JP(condition, HLOrImmediate::Immediate(address)),
                    )
                }
                4 | 6 => {
                    let direction = match y {
                        4 => Direction::FromA,
                        6 => Direction::IntoA,
                        _ => unreachable!(),
                    };

                    (
                        i,
                        Instruction::Ld(LoadType::LastByteAddress(COrImmediate::C, direction)),
                    )
                }
                5 | 7 => {
                    let direction = match y {
                        5 => Direction::FromA,
                        7 => Direction::IntoA,
                        _ => unreachable!(),
                    };
                    let (i, address) = le_u16().parse(i)?;
                    (
                        i,
                        Instruction::Ld(LoadType::Indirect(
                            LoadIndirect::Immediate(address),
                            direction,
                        )),
                    )
                }
                _ => nyi(),
            },
            3 => match y {
                0 => {
                    let (i, address) = le_u16().parse(i)?;
                    (
                        i,
                        Instruction::JP(JumpTest::Always, HLOrImmediate::Immediate(address)),
                    )
                }
                1 => prefixed_instruction(i)?,
                6 => (i, Instruction::Di),
                7 => (i, Instruction::Ei),
                _ => panic!("non-existent instruction: {}", unreachable()),
            },
            4 => match y {
                0..=3 => {
                    let (i, address) = le_u16().parse(i)?;
                    let condition = JumpTest::from_u8(y).unwrap();
                    (i, Instruction::Call(condition, address))
                }
                _ => panic!("non-existent instruction: {}", unreachable()),
            },
            5 => match q {
                0 => {
                    let reg = Register16Alt::from_u8(p).unwrap();
                    (i, Instruction::Push(reg))
                }
                1 => match p {
                    0 => {
                        let (i, address) = le_u16().parse(i)?;
                        (i, Instruction::Call(JumpTest::Always, address))
                    }
                    _ => panic!("non-existent instruction: {}", unreachable()),
                },
                _ => unreachable!("{}", unreachable()),
            },
            6 => {
                let alu = Alu::from_u8(y).unwrap();
                let (i, value) = le_u8().parse(i)?;
                (
                    i,
                    Instruction::Arithmetic(alu, RegisterOrImmediate::Immediate(value)),
                )
            }
            7 => {
                let address = u16::from(y) * 8;
                (i, Instruction::Reset(address))
            }
            _ => unreachable!("{}", unreachable()),
        },
        _ => unreachable!("{}", unreachable()),
    })
}

fn prefixed_instruction(i: &[u8]) -> IResult<&[u8], Instruction> {
    let (i, (x, y, z)) = bits::bits::<_, (u8, u8, u8), Error<_>, _, _>((
        bits::complete::take(2usize),
        bits::complete::take(3usize),
        bits::complete::take(3usize),
    ))
    .parse(i)?;
    assert!(x < 4);
    assert!(y < 8);
    assert!(z < 8);

    let unreachable = || {
        format!("impossible prefixed state! X:{x} Z:{z} Y:{y}\ndid you increment pc incorrectly?")
    };

    let reg = Register::from_u8(z).unwrap();
    Ok(match x {
        0 => {
            let rot = Rot::from_u8(y).unwrap();
            (i, Instruction::Rot(rot, reg))
        }
        1 => (i, Instruction::Bit(y, reg)),
        2 => (i, Instruction::Res(y, reg)),
        3 => (i, Instruction::Set(y, reg)),
        _ => unreachable!("{}", unreachable()),
    })
}
