use instruction::*;
use nom::{
    IResult, Parser, bits,
    bytes::complete::take,
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

    Ok(match x {
        0 => match z {
            1 => match q {
                0 => {
                    let (i, target) = le_u16().parse(i)?;
                    let reg = RegisterPairsSP::from_u8(p).expect("p should be within range 0-3");

                    (
                        i,
                        Instruction::Ld(LoadType::Word(reg, LoadWordSource::Immediate(target))),
                    )
                }
                1 => nyi(),
                _ => unreachable!(),
            },
            _ => nyi(),
        },
        1..3 => nyi(),
        _ => unreachable!(),
    })
}
