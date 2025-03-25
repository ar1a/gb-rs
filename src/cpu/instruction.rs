pub enum Instruction {
    Add(ArithmeticTarget),
}

pub enum ArithmeticTarget {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
}
