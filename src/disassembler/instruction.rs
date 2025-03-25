#![allow(dead_code)]
#[derive(Debug)]
pub enum Instruction {
    Add(ArithmeticTarget),
    Ld(LoadType),
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
