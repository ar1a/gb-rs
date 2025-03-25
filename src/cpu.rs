#![allow(dead_code)]
use enumflags2::make_bitflags;
use registers::*;

use crate::disassembler::{instruction::*, parse_instruction};

pub mod memorybus;
pub mod registers;

#[derive(Debug, Default)]
struct Cpu {
    registers: Registers,
    /// The Program Counter register
    pc: u16,
    sp: u16,
    bus: memorybus::MemoryBus,
}

impl Cpu {
    fn step(&mut self) {
        let slice = self.bus.slice_from(self.pc);
        let (_, instruction) = parse_instruction(slice).unwrap();
        eprintln!("read opcode {:#x} at {:#x}", slice[0], self.pc);
        let next_pc = self.execute(instruction);

        self.pc = next_pc;
    }

    fn execute(&mut self, instruction: Instruction) -> u16 {
        #![allow(unreachable_patterns)]
        #![allow(clippy::infallible_destructuring_match)]
        match instruction {
            Instruction::Ld(load_type) => match load_type {
                LoadType::Word(target, source) => {
                    let source_value = match source {
                        LoadWordSource::Value(x) => x,
                    };
                    match target {
                        LoadWordTarget::SP => self.sp = source_value,
                        LoadWordTarget::BC => self.registers.set_bc(source_value),
                        LoadWordTarget::DE => self.registers.set_de(source_value),
                        LoadWordTarget::HL => self.registers.set_hl(source_value),
                    };
                    eprintln!("  {:?} = {:#4x}", target, source_value);
                    match source {
                        LoadWordSource::Value(_) => self.pc.wrapping_add(3),
                    }
                }
            },
            Instruction::Add(target) => match target {
                // FIXME: abstract this like the others
                ArithmeticTarget::C => {
                    let value = self.registers.c;
                    let new_value = self.add(value);
                    self.registers.a = new_value;
                    self.pc.wrapping_add(1)
                }
                _ => todo!("unimplemented target: {:?}", target),
            },
            // Adc
            // Sub
            // Sbc
            // And
            // Or
            Instruction::Xor(source) => {
                let value = match source {
                    XorSource::A => self.registers.a,
                    XorSource::B => self.registers.b,
                    XorSource::C => self.registers.c,
                    XorSource::D => self.registers.d,
                    XorSource::E => self.registers.e,
                    XorSource::L => self.registers.l,
                    XorSource::HL => self.bus.read_byte(self.registers.hl()),
                    XorSource::Value(x) => x,
                };
                self.registers.a = self.xor(value);
                eprintln!("  A ^= {:?} = {:#x}", source, self.registers.a);
                match source {
                    XorSource::Value(_) => self.pc.wrapping_add(2),
                    _ => self.pc.wrapping_add(1),
                }
            }
            // Cp
            // Inc
            // Dec
            _ => todo!("unimplemented instruction: {:?}", instruction),
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

    fn xor(&mut self, value: u8) -> u8 {
        let new_value = self.registers.a ^ value;
        let flags = &mut self.registers.f;
        flags.set(Flags::Zero, new_value == 0);
        flags.remove(make_bitflags!(Flags::{Subtraction | Carry | HalfCarry}));
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

    #[test]
    fn test_boot_rom() {
        let boot_rom = include_bytes!("../dmg_boot.bin");
        let mut cpu = Cpu::default();
        cpu.bus.slice_mut()[0..256].copy_from_slice(boot_rom);
        loop {
            cpu.step();
        }
    }
}
