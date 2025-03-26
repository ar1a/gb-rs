#![allow(dead_code)]
use enumflags2::make_bitflags;
use memorybus::*;
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
    bus: MemoryBus,
}

impl Cpu {
    fn step(&mut self) {
        eprintln!();
        let slice = self.bus.slice_from(self.pc);
        let (_, instruction) = parse_instruction(slice).unwrap();
        if slice[0] == 0xcb {
            eprintln!(
                "read opcode {:#4x} at {:#4x}",
                // big endian so the opcode is printed in the order its read
                u16::from_be_bytes(slice[0..2].try_into().unwrap()),
                self.pc
            );
        } else {
            eprintln!("read opcode {:#x} at {:#4x}", slice[0], self.pc);
        }
        let next_pc = self.execute(instruction);
        eprintln!("{}", self.format_state()); // TODO: Log to a file instead

        self.pc = next_pc;
    }

    fn format_state(&self) -> String {
        format!(
            "A:{:02X} F:{} B:{:02X} C:{:02X} D:{:02X} E:{:02X} H:{:02X} L:{:02X} SP:{:04X} PC:{:04X} PCMEM:{:02X},{:02X},{:02X},{:02X}",
            self.registers.a,
            self.registers.f,
            self.registers.b,
            self.registers.c,
            self.registers.d,
            self.registers.e,
            self.registers.h,
            self.registers.l,
            self.sp,
            self.pc,
            self.bus.read_byte(self.pc),
            self.bus.read_byte(self.pc + 1),
            self.bus.read_byte(self.pc + 2),
            self.bus.read_byte(self.pc + 3),
        )
    }

    fn execute(&mut self, instruction: Instruction) -> u16 {
        #![allow(unreachable_patterns)]
        #![allow(clippy::infallible_destructuring_match)]
        match instruction {
            Instruction::Ld(load_type) => match load_type {
                LoadType::Indirect(indirect_type, direction) => {
                    let address = match indirect_type {
                        LoadIndirect::BC => self.registers.bc(),
                        LoadIndirect::DE => self.registers.de(),
                        LoadIndirect::HLDec | LoadIndirect::HLInc => self.registers.hl(),
                    };

                    match direction {
                        Direction::IntoA => self.registers.a = self.bus.read_byte(address),
                        Direction::FromA => self.bus.write_byte(address, self.registers.a),
                    };

                    let adjust = match indirect_type {
                        LoadIndirect::HLDec => -1,
                        LoadIndirect::HLInc => 1,
                        _ => 0,
                    };
                    if adjust != 0 {
                        self.registers.set_hl(address.wrapping_add_signed(adjust));
                    }
                    eprintln!("  LD Indirect {:#?} {:?}", indirect_type, direction);
                    self.pc.wrapping_add(1)
                }
                LoadType::Byte(register, source) => {
                    let value = match source {
                        RegisterOrImmediate::Immediate(x) => x,
                        RegisterOrImmediate::Register(reg) => self.match_register(reg),
                    };
                    self.write_register(register, value);
                    eprintln!("  {:?} = {:#2x}", register, value);
                    match source {
                        RegisterOrImmediate::Immediate(_) => self.pc.wrapping_add(2),
                        _ => self.pc.wrapping_add(1),
                    }
                }
                LoadType::Word(target, source) => {
                    let source_value = match source {
                        LoadWordSource::Immediate(x) => x,
                    };
                    match target {
                        RegisterPairsSP::SP => self.sp = source_value,
                        RegisterPairsSP::BC => self.registers.set_bc(source_value),
                        RegisterPairsSP::DE => self.registers.set_de(source_value),
                        RegisterPairsSP::HL => self.registers.set_hl(source_value),
                    };
                    eprintln!("  {:?} = {:#4x}", target, source_value);
                    match source {
                        LoadWordSource::Immediate(_) => self.pc.wrapping_add(3),
                    }
                }
                LoadType::LastByteAddress(source, direction) => {
                    let offset = match source {
                        COrImmediate::C => self.registers.c,
                        COrImmediate::Immediate(x) => x,
                    };
                    let address = 0xFF00 + offset as u16;

                    match direction {
                        Direction::FromA => {
                            eprintln!("  *({:#4x}) = A = {:#2x}", address, self.registers.a);
                            self.bus.write_byte(address, self.registers.a)
                        }
                        Direction::IntoA => {
                            eprintln!("  A = *({:#4x}) = {:#2x}", address, self.registers.a);
                            self.registers.a = self.bus.read_byte(address);
                        }
                    };

                    match source {
                        COrImmediate::Immediate(_) => self.pc.wrapping_add(2),
                        _ => self.pc.wrapping_add(1),
                    }
                }
            },
            Instruction::Arithmetic(alu, source) => match alu {
                Alu::Xor => {
                    let value = match source {
                        RegisterOrImmediate::Register(register) => self.match_register(register),
                        RegisterOrImmediate::Immediate(_value) => todo!(),
                    };
                    self.registers.a = self.xor(value);
                    eprintln!("  A ^= {:?} = {:#x}", source, self.registers.a);
                    match source {
                        RegisterOrImmediate::Immediate(_) => self.pc.wrapping_add(2),
                        _ => self.pc.wrapping_add(1),
                    }
                }
                _ => todo!("alu opertion: {:?} {:?}", alu, source),
            },
            Instruction::Bit(bit, source) => {
                let value = self.match_register(source);
                let mask = 1 << bit;
                self.bit(mask, value);
                eprintln!(
                    "  {:?} {:#2x} (0b{:0>8b}) & 0b{:0>8b} = {}",
                    source,
                    value,
                    value,
                    mask,
                    !self.registers.f.contains(Flags::Zero)
                );
                self.pc.wrapping_add(2)
            }
            Instruction::JR(condition, relative) => {
                let should_jump = match condition {
                    JumpTest::NotZero => !self.registers.f.contains(Flags::Zero),
                    JumpTest::Zero => self.registers.f.contains(Flags::Zero),
                    JumpTest::NotCarry => !self.registers.f.contains(Flags::Carry),
                    JumpTest::Carry => self.registers.f.contains(Flags::Carry),
                    JumpTest::Always => true,
                };
                eprintln!(
                    "  relative jump of {} if {:?} (will jump: {})",
                    relative, condition, should_jump
                );
                self.relative_jump(should_jump, relative)
            }
            Instruction::Inc(register) => {
                let inc = self.inc(self.match_register(register));
                self.write_register(register, inc);
                eprintln!("  INC {:?} = {:#02x}", register, inc);
                self.pc.wrapping_add(1)
            }
            _ => todo!("unimplemented instruction: {:?}", instruction),
        }
    }

    fn match_register(&self, register: Register) -> u8 {
        match register {
            Register::A => self.registers.a,
            Register::B => self.registers.b,
            Register::C => self.registers.c,
            Register::D => self.registers.d,
            Register::E => self.registers.e,
            Register::L => self.registers.l,
            Register::H => self.registers.h,
            Register::HLIndirect => self.bus.read_byte(self.registers.hl()),
        }
    }

    fn write_register(&mut self, register: Register, value: u8) {
        match register {
            Register::A => self.registers.a = value,
            Register::B => self.registers.b = value,
            Register::C => self.registers.c = value,
            Register::D => self.registers.d = value,
            Register::E => self.registers.e = value,
            Register::L => self.registers.l = value,
            Register::H => self.registers.h = value,
            Register::HLIndirect => self.bus.write_byte(self.registers.hl(), value),
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

    fn relative_jump(&mut self, should_jump: bool, offset: i8) -> u16 {
        let pc = self.pc.wrapping_add(2);
        if should_jump {
            pc.wrapping_add_signed(offset as i16)
        } else {
            pc
        }
    }

    fn inc(&mut self, value: u8) -> u8 {
        let (new_value, _overflow) = value.overflowing_add(1);
        let flags = &mut self.registers.f;
        flags.set(Flags::Zero, new_value == 0);
        flags.remove(Flags::Subtraction);
        // HalfCarry is set if the lower 4 bits added together don't fit in the lower 4 bits
        flags.set(Flags::HalfCarry, (value & 0b1111) + 1 > 0b1111);
        new_value
    }
}

#[cfg(test)]
mod test {
    use super::*;

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
