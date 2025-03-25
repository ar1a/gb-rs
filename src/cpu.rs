#![allow(dead_code)]
mod registers;
use registers::*;

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
