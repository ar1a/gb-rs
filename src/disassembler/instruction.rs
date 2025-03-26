#![allow(dead_code)]
use num_derive::FromPrimitive;

#[derive(Debug)]
pub enum Instruction {
    Ld(LoadType),
    Arithmetic(Alu, Registers8Bit),
    Bit(u8, BitSource),
    JR(JumpTest, i8),
}

#[derive(Debug)]
pub enum LoadType {
    Indirect(LoadIndirect, Direction),
    Byte(LoadByteTarget, LoadByteSource),
    Word(RegisterPairsSP, LoadWordSource),
    COffset(LoadCOffsetSource),
}

#[derive(Debug)]
pub enum LoadByteTarget {
    A,
    B,
    C,
    D,
    H,
    L,
    HL,
}

#[derive(Debug)]
pub enum LoadIndirect {
    BC,
    DE,
    HLDec,
    HLInc,
}

#[derive(Debug)]
pub enum Direction {
    FromA,
    IntoA,
}

#[derive(Debug)]
pub enum LoadByteSource {
    Value(u8),
}

#[derive(Debug)]
pub enum LoadWordSource {
    Immediate(u16),
}

#[derive(Debug)]
pub enum LoadCOffsetSource {
    C,
    A,
}

#[derive(Debug)]
pub enum LoadByteDecTarget {
    A,
    HL,
}

#[derive(Debug)]
pub enum BitSource {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
    HL,
}

#[derive(Debug)]
pub enum JumpTest {
    NotZero,
    Zero,
    NotCarry,
    Carry,
    Always,
}

#[derive(Debug, FromPrimitive)]
pub enum Registers8Bit {
    B,
    C,
    D,
    E,
    H,
    L,
    HLIndirect,
    A,
}

#[derive(Debug, FromPrimitive)]
pub enum RegisterPairsSP {
    BC,
    DE,
    HL,
    SP,
}
#[derive(Debug, FromPrimitive)]
pub enum RegisterPairsAF {
    BC,
    DE,
    HL,
    AF,
}

#[derive(Debug, FromPrimitive)]
pub enum Alu {
    Add,
    Adc,
    Sub,
    Sbc,
    And,
    Xor,
    Or,
    Cp,
}

#[derive(Debug, FromPrimitive)]
pub enum Rot {
    Rlc,
    Rrc,
    Rl,
    Rr,
    Sla,
    Sra,
    Swap,
    Srl,
}
