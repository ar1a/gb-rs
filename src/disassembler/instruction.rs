#![allow(dead_code)]
use num_derive::FromPrimitive;
use parse_display::Display;

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
    Ret,
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

#[derive(Debug, Clone, Copy, Display)]
pub enum RegisterOrImmediate {
    #[display("{0}")]
    Register(Register),
    #[display("{0:02X}")]
    Immediate(u8),
}

#[derive(Debug, Clone, Copy, Display)]
pub enum COrImmediate {
    C,
    #[display("{0:02X}")]
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

#[derive(Debug, Clone, Copy, Display)]
pub enum LoadIndirect {
    BC,
    DE,
    #[display("HL")]
    HLDec,
    #[display("HL")]
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

#[derive(Debug, FromPrimitive, Clone, Copy, Display)]
pub enum JumpTest {
    #[display("NZ,")]
    NotZero,
    #[display("Z,")]
    Zero,
    #[display("NC,")]
    NotCarry,
    #[display("C,")]
    Carry,
    // Not possible in DGM using FromPrimitive
    #[display("")]
    Always,
}

#[derive(Debug, FromPrimitive, Clone, Copy, Display)]
pub enum Register {
    B,
    C,
    D,
    E,
    H,
    L,
    #[display("(HL)")]
    HLIndirect,
    A,
}

#[derive(Debug, FromPrimitive, Clone, Copy, Display)]
pub enum Register16 {
    BC,
    DE,
    HL,
    SP,
}
#[derive(Debug, FromPrimitive, Clone, Copy, Display)]
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
