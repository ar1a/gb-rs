#![allow(dead_code)]
use registers::*;

use crate::disassembler::parse_instruction;

pub mod instruction;
pub mod memorybus;
pub mod registers;

#[derive(Debug, Default)]
struct Cpu {
    registers: Registers,
    /// The Program Counter register
    pc: u16,
    bus: memorybus::MemoryBus,
}

impl Cpu {
    fn step(&mut self) {
        let (_, instruction) =
            dbg!(parse_instruction(&self.bus.memory[self.pc as usize..]).unwrap());
        eprintln!(
            "read opcode {:#x} at {:#x}",
            self.bus.memory[self.pc as usize], self.pc
        );
        let next_pc = self.execute(instruction);

        self.pc = next_pc;
    }

    fn execute(&mut self, instruction: instruction::Instruction) -> u16 {
        match instruction {
            instruction::Instruction::Add(target) => match target {
                instruction::ArithmeticTarget::C => {
                    let value = self.registers.c;
                    let new_value = self.add(value);
                    self.registers.a = new_value;
                    self.pc.wrapping_add(1)
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

        cpu.execute(instruction::Instruction::Add(
            instruction::ArithmeticTarget::C,
        ));
        assert_eq!(cpu.registers.a, 255);
        assert!(!cpu.registers.f.contains(Flags::Carry));

        cpu.execute(instruction::Instruction::Add(
            instruction::ArithmeticTarget::C,
        ));
        assert_eq!(cpu.registers.a, 0);
        assert!(cpu.registers.f.contains(Flags::Carry));
    }

    #[test]
    fn test_boot_rom() {
        let boot_rom = include_bytes!("../dmg_boot.bin");
        let mut cpu = Cpu::default();
        cpu.bus.memory[0..256].copy_from_slice(boot_rom);
        loop {
            cpu.step();
        }
    }
}
