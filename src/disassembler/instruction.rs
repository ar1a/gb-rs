#![allow(dead_code)]
use num_derive::FromPrimitive;

#[derive(Debug, Clone, Copy)]
pub enum Instruction {
    Ld(LoadType),
    Arithmetic(Alu, RegisterOrImmediate),
    Bit(u8, Register),
    JR(JumpTest, i8),
    Inc(Register),
    Inc16(Register16),
    Dec(Register),
    Dec16(Register16),
    Call(JumpTest, u16),
    Push(Register16Alt),
    Pop(Register16Alt),
    Rot(Rot, Register),
    Rlca,
    Rrca,
    Rla,
    Rra,
}

#[derive(Debug, Clone, Copy)]
pub enum LoadType {
    Indirect(LoadIndirect, Direction),
    Byte(Register, RegisterOrImmediate),
    Word(Register16, LoadWordSource),
    LastByteAddress(COrImmediate, Direction),
}

#[derive(Debug, Clone, Copy)]
pub enum RegisterOrImmediate {
    Register(Register),
    Immediate(u8),
}

#[derive(Debug, Clone, Copy)]
pub enum COrImmediate {
    C,
    Immediate(u8),
}

#[derive(Debug, Clone, Copy)]
pub enum LoadByteTarget {
    A,
    B,
    C,
    D,
    H,
    L,
    HL,
}

#[derive(Debug, Clone, Copy)]
pub enum LoadIndirect {
    BC,
    DE,
    HLDec,
    HLInc,
}

#[derive(Debug, Clone, Copy)]
pub enum Direction {
    FromA,
    IntoA,
}

#[derive(Debug, Clone, Copy)]
pub enum LoadByteSource {
    Value(u8),
}

#[derive(Debug, Clone, Copy)]
pub enum LoadWordSource {
    Immediate(u16),
}

#[derive(Debug, Clone, Copy)]
pub enum LoadByteDecTarget {
    A,
    HL,
}

#[derive(Debug, FromPrimitive, Clone, Copy)]
pub enum JumpTest {
    NotZero,
    Zero,
    NotCarry,
    Carry,
    // Not possible in DGM using FromPrimitive
    Always,
}

#[derive(Debug, FromPrimitive, Clone, Copy)]
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

#[derive(Debug, FromPrimitive, Clone, Copy)]
pub enum Register16 {
    BC,
    DE,
    HL,
    SP,
}
#[derive(Debug, FromPrimitive, Clone, Copy)]
pub enum Register16Alt {
    BC,
    DE,
    HL,
    AF,
}

#[derive(Debug, FromPrimitive, Clone, Copy)]
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

#[derive(Debug, FromPrimitive, Clone, Copy)]
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
