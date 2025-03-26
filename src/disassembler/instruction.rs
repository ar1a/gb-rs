#![allow(dead_code)]
use num_derive::FromPrimitive;

#[derive(Debug)]
pub enum Instruction {
    Ld(LoadType),
    Arithmetic(Alu, RegisterOrImmediate),
    Bit(u8, Register),
    JR(JumpTest, i8),
    Inc(Register),
}

#[derive(Debug)]
pub enum LoadType {
    Indirect(LoadIndirect, Direction),
    Byte(Register, RegisterOrImmediate),
    Word(RegisterPairsSP, LoadWordSource),
    LastByteAddress(COrImmediate, Direction),
}

#[derive(Debug)]
pub enum RegisterOrImmediate {
    Register(Register),
    Immediate(u8),
}

#[derive(Debug)]
pub enum COrImmediate {
    C,
    Immediate(u8),
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
pub enum LoadByteDecTarget {
    A,
    HL,
}

#[derive(Debug, FromPrimitive)]
pub enum JumpTest {
    NotZero,
    Zero,
    NotCarry,
    Carry,
    // Not possible in DGM using FromPrimitive
    Always,
}

#[derive(Debug, FromPrimitive, Copy, Clone)]
pub enum Register {
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
