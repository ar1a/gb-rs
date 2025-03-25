use nom::{IResult, Parser, bytes::complete::take};

use crate::cpu::instruction::{ArithmeticTarget, Instruction};

pub fn parse_instruction(i: &[u8]) -> IResult<&[u8], Instruction> {
    let (i, byte) = take(1usize).parse(i)?;
    Ok(match byte[0] {
        0x81 => (i, Instruction::Add(ArithmeticTarget::C)),
        _ => todo!(),
    })
}
