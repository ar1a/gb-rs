#![allow(dead_code)]
#[derive(Debug)]
pub enum Instruction {
    Ld(LoadType),
    Add(ArithmeticTarget),
    Xor(XorSource),
    Bit(u8, BitSource),
    JR(JumpTest, i8),
}

#[derive(Debug)]
pub enum ArithmeticTarget {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
}

#[derive(Debug)]
pub enum LoadType {
    ByteDec(LoadByteDecTarget),
    Byte(LoadByteTarget, LoadByteSource),
    Word(LoadWordTarget, LoadWordSource),
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
pub enum LoadByteSource {
    Value(u8),
}
#[derive(Debug)]
pub enum LoadWordTarget {
    BC,
    DE,
    HL,
    SP,
}

#[derive(Debug)]
pub enum LoadWordSource {
    Value(u16),
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
pub enum XorSource {
    A,
    B,
    C,
    D,
    E,
    L,
    HL,
    Value(u8),
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
