use instruction::*;
use nom::{IResult, Parser, bytes::complete::take, number::le_u16};

pub mod instruction;

pub fn parse_instruction(i: &[u8]) -> IResult<&[u8], Instruction> {
    let (i, byte) = take(1usize).parse(i)?;
    let byte = byte[0];
    Ok(match byte {
        // 8-Bit Loads
        // 16-Bit Loads
        0x31 => {
            let (i, target) = le_u16().parse(i)?;
            (
                i,
                Instruction::Ld(LoadType::Word(
                    LoadWordTarget::SP,
                    LoadWordSource::Value(target),
                )),
            )
        }
        // 8-Bit ALU
        // Add
        0x81 => (i, Instruction::Add(ArithmeticTarget::C)),
        // Adc
        // Sub
        // Sbc
        // And
        // Or
        // Xor
        0xAF => (i, Instruction::Xor(XorSource::A)),
        // Cp
        // Inc
        // Dec
        _ => todo!("Haven't implemented {byte:#x}"),
    })
}
