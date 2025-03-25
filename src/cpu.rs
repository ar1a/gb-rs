#![allow(dead_code)]
use enumflags2::{BitFlag as _, BitFlags, bitflags};

/// Base registers
#[derive(Debug, Default)]
struct Registers {
    a: u8,
    b: u8,
    c: u8,
    d: u8,
    e: u8,
    h: u8,
    l: u8,

    f: BitFlags<Flags>,
}

#[bitflags]
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq)]
enum Flags {
    /// Set if the result of an operation is zero
    Zero = 0b1000_0000,
    /// Set if previous instruction was a Subtraction
    Subtraction = 0b0100_0000,
    /// Set if lower 4 bits carried over to upper 4 bits
    HalfCarry = 0b0010_0000,
    /// Set when something overflows
    Carry = 0b0001_0000,
}

impl Registers {
    const fn bc(&self) -> u16 {
        u16::from_le_bytes([self.b, self.c])
    }
    const fn set_bc(&mut self, value: u16) {
        let [b, c] = value.to_le_bytes();
        self.b = b;
        self.c = c;
    }

    fn af(&self) -> u16 {
        u16::from_le_bytes([self.a, self.f.bits()])
    }
    fn set_af(&mut self, value: u16) {
        let [a, f] = value.to_le_bytes();
        self.a = a;
        self.f = Flags::from_bits(f).unwrap();
    }

    const fn de(&self) -> u16 {
        u16::from_le_bytes([self.d, self.e])
    }
    const fn set_de(&mut self, value: u16) {
        let [d, e] = value.to_le_bytes();
        self.d = d;
        self.e = e;
    }

    const fn hl(&self) -> u16 {
        u16::from_le_bytes([self.h, self.l])
    }
    const fn set_hl(&mut self, value: u16) {
        let [h, l] = value.to_le_bytes();
        self.h = h;
        self.l = l;
    }
}

enum Instruction {
    Add(ArithmeticTarget),
}

enum ArithmeticTarget {
    A,
    B,
    C,
    D,
    E,
    H,
    L,
}

#[derive(Debug, Default)]
struct Cpu {
    registers: Registers,
}

impl Cpu {
    fn execute(&mut self, instruction: Instruction) {
        match instruction {
            Instruction::Add(target) => match target {
                ArithmeticTarget::C => {
                    let value = self.registers.c;
                    let new_value = self.add(value);
                    self.registers.a = new_value;
                }
                _ => todo!("support more targets"),
            },
        }
    }

    fn add(&mut self, value: u8) -> u8 {
        let (new_value, overflow) = self.registers.a.overflowing_add(value);
        let flags = &mut self.registers.f;
        flags.set(Flags::Zero, new_value == 0);
        flags.remove(Flags::Subtraction);
        flags.set(Flags::Carry, overflow);
        // HalfCarry is set if the lower 4 bits added together don't fit in the lower 4 bits
        flags.set(
            Flags::HalfCarry,
            (self.registers.a & 0b1111) + (value & 0b1111) > 0b1111,
        );
        new_value
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_add() {
        let mut cpu = Cpu::default();
        cpu.registers.a = u8::MAX - 1;
        cpu.registers.c = 1;

        cpu.execute(Instruction::Add(ArithmeticTarget::C));
        assert_eq!(cpu.registers.a, 255);
        assert!(!cpu.registers.f.contains(Flags::Carry));

        cpu.execute(Instruction::Add(ArithmeticTarget::C));
        assert_eq!(cpu.registers.a, 0);
        assert!(cpu.registers.f.contains(Flags::Carry));
    }
}
