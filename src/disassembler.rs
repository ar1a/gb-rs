use instruction::*;
use nom::{
    IResult, Parser,
    bytes::complete::take,
    number::{le_i8, le_u16},
};

pub mod instruction;

pub fn parse_instruction(i: &[u8]) -> IResult<&[u8], Instruction> {
    let (i, byte) = take(1usize).parse(i)?;
    let byte = byte[0];
    Ok(match byte {
        // 8-Bit Loads
        0x32 => (i, Instruction::Ld(LoadType::ByteDec(LoadByteDecTarget::HL))),
        // 16-Bit Loads
        0x21 => {
            let (i, source) = le_u16().parse(i)?;
            (
                i,
                Instruction::Ld(LoadType::Word(
                    LoadWordTarget::HL,
                    LoadWordSource::Value(source),
                )),
            )
        }
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
        0xCB => {
            let (i, byte) = take(1usize).parse(i)?;
            let byte = byte[0];
            match byte {
                0x7c => (i, Instruction::Bit(1 << 7, BitSource::H)),
                _ => todo!("Haven't implemented prefixed {:#x} {byte:#x}", 0xCB),
            }
        }

        // Jumps
        0x20 => {
            let (i, address) = le_i8().parse(i)?;
            (i, Instruction::JR(JumpTest::NotZero, address))
        }
        _ => todo!("Haven't implemented {byte:#x}"),
    })
}
