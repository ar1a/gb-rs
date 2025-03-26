use instruction::*;
use nom::{
    IResult, Parser, bits,
    error::Error,
    number::{le_i8, le_u8, le_u16},
};
use num_traits::FromPrimitive as _;

pub mod instruction;

pub fn parse_instruction(i: &[u8]) -> IResult<&[u8], Instruction> {
    // based on <https://gb-archive.github.io/salvage/decoding_gbz80_opcodes/Decoding%20Gamboy%20Z80%20Opcodes.html>
    let (i, (x, y, z)) = bits::bits::<_, (u8, u8, u8), Error<_>, _, _>((
        bits::complete::take(2usize),
        bits::complete::take(3usize),
        bits::complete::take(3usize),
    ))
    .parse(i)?;
    assert!(x < 4);
    assert!(y < 8);
    assert!(z < 8);
    let p = y >> 1;
    let q = y % 2;

    let nyi = || todo!("instruction parse for X:{x} Z:{z} Y:{y} P:{p} Q:{q}");
    let unreachable = format!(
        "impossible state! X:{x} Z:{z} Y:{y} P:{p} Q:{q}\ndid you increment pc incorrectly?"
    );

    Ok(match x {
        0 => match z {
            0 => match y {
                4..7 => {
                    let condition = JumpTest::from_u8(y - 4).unwrap();
                    let (i, relative) = le_i8().parse(i)?;
                    (i, Instruction::JR(condition, relative))
                }
                _ => nyi(),
            },
            1 => match q {
                0 => {
                    let (i, target) = le_u16().parse(i)?;
                    let reg = RegisterPairsSP::from_u8(p).unwrap();

                    (
                        i,
                        Instruction::Ld(LoadType::Word(reg, LoadWordSource::Immediate(target))),
                    )
                }
                1 => nyi(),
                _ => unreachable!("{}", unreachable),
            },
            2 => {
                let direction = match q {
                    0 => Direction::FromA,
                    1 => Direction::IntoA,
                    _ => unreachable!("{}", unreachable),
                };
                let indirect_type = match p {
                    0 => LoadIndirect::BC,
                    1 => LoadIndirect::DE,
                    2 => LoadIndirect::HLInc,
                    3 => LoadIndirect::HLDec,
                    _ => unreachable!("{}", unreachable),
                };
                (
                    i,
                    Instruction::Ld(LoadType::Indirect(indirect_type, direction)),
                )
            }
            4 => {
                let reg = Register::from_u8(y).unwrap();
                (i, Instruction::Inc(reg))
            }
            6 => {
                let reg = Register::from_u8(y).unwrap();
                let (i, value) = le_u8().parse(i)?;
                (
                    i,
                    Instruction::Ld(LoadType::Byte(reg, RegisterOrImmediate::Immediate(value))),
                )
            }
            _ => nyi(),
        },
        1 => match z {
            1..6 | 7 => {
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
            6 => nyi(),
            _ => unreachable!("{}", unreachable),
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
                _ => nyi(),
            },
            2 => match y {
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
                _ => nyi(),
            },
            3 => match y {
                1 => prefixed_instruction(i)?,
                _ => nyi(),
            },
            5 => match q {
                0 => nyi(),
                1 => match p {
                    0 => {
                        let (i, address) = le_u16().parse(i)?;
                        (i, Instruction::Call(JumpTest::Always, address))
                    }
                    _ => panic!("non-existent instruction: {}", unreachable),
                },
                _ => unreachable!("{}", unreachable),
            },
            _ => nyi(),
        },
        _ => unreachable!("{}", unreachable),
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

    let nyi = || todo!("prefixed instruction parse for X:{x} Z:{z} Y:{y}");
    let unreachable =
        format!("impossible prefixed state! X:{x} Z:{z} Y:{y}\ndid you increment pc incorrectly?");

    Ok(match x {
        1 => {
            let reg = Register::from_u8(z).unwrap();
            (i, Instruction::Bit(y, reg))
        }
        _ => nyi(),
    })
}
