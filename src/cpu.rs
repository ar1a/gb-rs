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
        if slice[0] == 0xcb {
            eprintln!(
                "read opcode {:#4x} at {:#x}",
                // big endian so the opcode is printed in the order its read
                u16::from_be_bytes(slice[0..2].try_into().unwrap()),
                self.pc
            );
        } else {
            eprintln!("read opcode {:#x} at {:#x}", slice[0], self.pc);
        }
        let next_pc = self.execute(instruction);

        self.pc = next_pc;
    }

    fn execute(&mut self, instruction: Instruction) -> u16 {
        #![allow(unreachable_patterns)]
        #![allow(clippy::infallible_destructuring_match)]
        match instruction {
            Instruction::Ld(load_type) => match load_type {
                LoadType::ByteDec(target) => {
                    match target {
                        LoadByteDecTarget::A => {
                            self.registers.a = self.bus.read_byte(self.registers.hl());
                            eprintln!(
                                "  {:?} = *({:#4x}) = {:#x}",
                                target,
                                self.registers.hl(),
                                self.registers.a
                            );
                        }
                        LoadByteDecTarget::HL => {
                            self.bus.write_byte(self.registers.hl(), self.registers.a);
                            eprintln!(
                                "  *({:?}) = {:#x} ({:?} is {:#4x})",
                                target,
                                self.registers.a,
                                target,
                                self.registers.hl(),
                            );
                        }
                    };
                    self.registers.set_hl(self.registers.hl() - 1);
                    eprintln!("  HL = {:#4x}", self.registers.hl());
                    self.pc.wrapping_add(1)
                }
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
            Instruction::Bit(mask, source) => {
                let value = match source {
                    BitSource::A => self.registers.a,
                    BitSource::B => self.registers.b,
                    BitSource::C => self.registers.c,
                    BitSource::D => self.registers.d,
                    BitSource::E => self.registers.e,
                    BitSource::H => self.registers.h,
                    BitSource::L => self.registers.l,
                    BitSource::HL => self.bus.read_byte(self.registers.hl()),
                };
                self.bit(mask, value);
                eprintln!(
                    "  {:?} {:#x} & {:#8b} = {}",
                    source,
                    value,
                    mask,
                    self.registers.f.contains(Flags::Zero)
                );
                self.pc.wrapping_add(2)
            }
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

    fn bit(&mut self, mask: u8, value: u8) {
        let flags = &mut self.registers.f;
        flags.set(Flags::Zero, value & mask == 0);
        flags.remove(Flags::Subtraction);
        flags.insert(Flags::HalfCarry);
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
