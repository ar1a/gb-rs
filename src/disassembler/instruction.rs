#![allow(dead_code)]
#[derive(Debug)]
pub enum Instruction {
    Ld(LoadType),
    Add(ArithmeticTarget),
    Xor(XorSource),
    Bit(u8, BitSource),
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
    Word(LoadWordTarget, LoadWordSource),
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
