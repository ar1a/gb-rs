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
            "A:{:02X} F:{:02X} B:{:02X} C:{:02X} D:{:02X} E:{:02X} H:{:02X} L:{:02X} SP:{:04X} PC:{:04X} PCMEM:{:02X},{:02X},{:02X},{:02X}",
            self.registers.a,
            self.registers.f.bits(),
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
                LoadType::Byte(target, source) => {
                    let source_value = match source {
                        LoadByteSource::Value(x) => x,
                    };
                    match target {
                        LoadByteTarget::A => self.registers.a = source_value,
                        LoadByteTarget::B => self.registers.b = source_value,
                        LoadByteTarget::C => self.registers.c = source_value,
                        LoadByteTarget::D => self.registers.d = source_value,
                        LoadByteTarget::H => self.registers.h = source_value,
                        LoadByteTarget::L => self.registers.l = source_value,
                        LoadByteTarget::HL => {
                            self.bus.write_byte(self.registers.hl(), source_value)
                        }
                    }
                    eprintln!("  {:?} = {:#2x}", target, source_value);
                    match source {
                        LoadByteSource::Value(_) => self.pc.wrapping_add(2),
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
                LoadType::COffset(source) => {
                    let address = 0xFF00 + self.registers.c as u16;
                    match source {
                        LoadCOffsetSource::C => {
                            self.registers.a = self.bus.read_byte(address);
                            eprintln!("  A = *({:#4x}) = {:#2x}", address, self.registers.a);
                        }
                        LoadCOffsetSource::A => {
                            self.bus.write_byte(address, self.registers.a);
                            eprintln!("  *({:#4x}) = A = {:#2x}", address, self.registers.a);
                        }
                    };

                    self.pc.wrapping_add(1)
                }
            },
            Instruction::Arithmetic(alu, source) => match alu {
                Alu::Xor => {
                    let value = match source {
                        RegisterOrImmediate::Register(ref register) => match register {
                            Register::A => self.registers.a,
                            Register::B => self.registers.b,
                            Register::C => self.registers.c,
                            Register::D => self.registers.d,
                            Register::E => self.registers.e,
                            Register::L => self.registers.l,
                            Register::H => self.registers.h,
                            Register::HLIndirect => self.bus.read_byte(self.registers.hl()),
                        },
                        RegisterOrImmediate::Immediate(value) => todo!(),
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
            // Instruction::Add(target) => match target {
            //     // FIXME: abstract this like the others
            //     ArithmeticTarget::C => {
            //         let value = self.registers.c;
            //         let new_value = self.add(value);
            //         self.registers.a = new_value;
            //         self.pc.wrapping_add(1)
            //     }
            //     _ => todo!("unimplemented target: {:?}", target),
            // },
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

    fn relative_jump(&mut self, should_jump: bool, offset: i8) -> u16 {
        let pc = self.pc.wrapping_add(2);
        if should_jump {
            pc.wrapping_add_signed(offset as i16)
        } else {
            pc
        }
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
