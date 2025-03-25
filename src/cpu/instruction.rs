#[derive(Debug)]
pub enum Instruction {
    Add(ArithmeticTarget),
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
