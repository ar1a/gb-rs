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
    debug_bytes_consumed: Vec<u8>,
    // Optionally used
    debug_context: Vec<String>,
}

impl Cpu {
    fn step(&mut self) {
        self.debug_context.clear();
        let slice = self.bus.slice_from(self.pc);
        let (after, instruction) = parse_instruction(slice).unwrap();
        let bytes_consumed_len = slice.len() - after.len();
        self.debug_bytes_consumed
            .splice(.., slice[..bytes_consumed_len].iter().copied());
        let next_pc = self.execute(instruction);
        // eprintln!("{}", self.format_state()); // TODO: Log to a file instead

        self.pc = next_pc;
    }

    fn format_state(&self) -> String {
        format!(
            "A:{:02X} F:{:0>4b} B:{:02X} C:{:02X} D:{:02X} E:{:02X} H:{:02X} L:{:02X} SP:{:04X} PC:{:04X} PCMEM:{:02X},{:02X},{:02X},{:02X}\nAF:{:04x} BC:{:04x} DE:{:04x} HL:{:04x}",
            self.registers.a,
            self.registers.f.bits() >> 4,
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
            self.registers.af(),
            self.registers.bc(),
            self.registers.de(),
            self.registers.hl()
        )
    }

    fn format_context(&self) -> String {
        self.debug_context
            .iter()
            .filter(|s| !s.is_empty())
            .cloned()
            .collect::<Vec<_>>()
            .join(", ")
    }

    fn print_debug(&self, opcode: &str, context: &str) {
        eprint!("{:04x}", self.pc);
        let bytes: String = self
            .debug_bytes_consumed
            .iter()
            .map(|byte| format!("{:02x} ", byte))
            .collect::<Vec<_>>()
            .join("");
        eprint!(" {:12}", bytes);
        eprint!("{:32}", opcode);
        eprintln!(" ; {}", context);
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
                        Direction::IntoA => {
                            let value = self.bus.read_byte(address);
                            self.print_debug(
                                &format!("LD A, ({:?})", indirect_type),
                                &format!("{} = {:04x}, A' = {:02x}", indirect_type, address, value),
                            );
                            self.registers.a = value;
                        }
                        Direction::FromA => {
                            let value = self.registers.a;
                            self.print_debug(
                                &format!("LD ({:?}), A", indirect_type),
                                &format!("{} = {:04x}, A = {:02x}", indirect_type, address, value),
                            );
                            self.bus.write_byte(address, value)
                        }
                    };

                    let adjust = match indirect_type {
                        LoadIndirect::HLDec => -1,
                        LoadIndirect::HLInc => 1,
                        _ => 0,
                    };
                    if adjust != 0 {
                        self.registers.set_hl(address.wrapping_add_signed(adjust));
                    }
                    self.pc.wrapping_add(1)
                }
                LoadType::Byte(register, source) => {
                    let value = match source {
                        RegisterOrImmediate::Immediate(x) => x,
                        RegisterOrImmediate::Register(reg) => {
                            let value = self.match_register(reg);
                            self.debug_context.push(format!("{} = {:02x}", reg, value));
                            value
                        }
                    };
                    self.print_debug(
                        &format!("LD {}, {}", register, source),
                        &self.format_context(),
                    );
                    self.write_register(register, value);
                    match source {
                        RegisterOrImmediate::Immediate(_) => self.pc.wrapping_add(2),
                        _ => self.pc.wrapping_add(1),
                    }
                }
                LoadType::Word(register, source) => {
                    let source_value = match source {
                        LoadWordSource::Immediate(x) => x,
                    };
                    self.write_register16(register, source_value);
                    self.print_debug(&format!("LD {}, {:02x}", register, source_value), "");
                    match source {
                        LoadWordSource::Immediate(_) => self.pc.wrapping_add(3),
                    }
                }
                LoadType::LastByteAddress(source, direction) => {
                    let offset = match source {
                        COrImmediate::C => {
                            self.debug_context
                                .push(format!("C = {:02x}", self.registers.c));
                            self.registers.c
                        }
                        COrImmediate::Immediate(x) => x,
                    };
                    let address = 0xFF00 + offset as u16;

                    match direction {
                        Direction::FromA => {
                            self.debug_context
                                .push(format!("A = {:02x}", self.registers.a));
                            self.print_debug(
                                &format!("LDH ({}), A", source),
                                &self.format_context(),
                            );
                            self.bus.write_byte(address, self.registers.a)
                        }
                        Direction::IntoA => {
                            let value = self.bus.read_byte(address);
                            self.debug_context
                                .push(format!("({:04x}) = {:02x}", address, value));
                            self.print_debug(
                                &format!("LDH A, ({})", source),
                                &self.format_context(),
                            );
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
                        RegisterOrImmediate::Register(register) => {
                            let value = self.match_register(register);
                            if !matches!(register, Register::A) {
                                self.debug_context
                                    .push(format!("{} = {:02x}", register, value));
                            }
                            value
                        }
                        RegisterOrImmediate::Immediate(value) => value,
                    };
                    self.debug_context
                        .push(format!("A = {:02x}", self.registers.a));
                    self.registers.a = self.xor(value);
                    self.print_debug(&format!("XOR {}", source), &self.format_context());
                    match source {
                        RegisterOrImmediate::Immediate(_) => self.pc.wrapping_add(2),
                        _ => self.pc.wrapping_add(1),
                    }
                }
                Alu::Cp => {
                    let value = match source {
                        RegisterOrImmediate::Register(register) => self.match_register(register),
                        RegisterOrImmediate::Immediate(value) => value,
                    };
                    self.cp(value);
                    eprintln!(
                        "  CP {:?} {:#02x} - {:#02x} == {:?}",
                        source, self.registers.a, value, self.registers.f
                    );

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
                let should_jump = self.match_jump_condition(condition);
                eprintln!(
                    "  relative jump of {} if {:?} (will jump: {})",
                    relative, condition, should_jump
                );
                self.relative_jump(should_jump, relative)
            }
            Instruction::Inc(register) => {
                let value = self.inc(self.match_register(register));
                self.write_register(register, value);
                eprintln!("  INC {:?}", register);
                self.pc.wrapping_add(1)
            }
            Instruction::Inc16(register) => {
                let value = self.match_register16(register).wrapping_add(1);
                self.write_register16(register, value);
                eprintln!("  INC {:?}", register);
                self.pc.wrapping_add(1)
            }
            Instruction::Dec(register) => {
                let value = self.dec(self.match_register(register));
                self.write_register(register, value);
                eprintln!("  DEC {:?}", register);
                self.pc.wrapping_add(1)
            }
            Instruction::Dec16(register) => {
                let value = self.match_register16(register).wrapping_sub(1);
                self.write_register16(register, value);
                eprintln!("  DEC {:?}", register);
                self.pc.wrapping_add(1)
            }
            Instruction::Call(condition, address) => {
                let should_jump = self.match_jump_condition(condition);
                eprintln!(
                    "  call to {:#04x} if {:?} (will jump: {})",
                    address, condition, should_jump
                );
                self.call(should_jump, address)
            }
            Instruction::Ret => {
                eprintln!("  RET to {:#04x}", self.bus.read_word(self.sp));
                self.retn(true)
            }
            Instruction::Push(register) => {
                let value = match register {
                    Register16Alt::BC => self.registers.bc(),
                    Register16Alt::DE => self.registers.de(),
                    Register16Alt::HL => self.registers.hl(),
                    Register16Alt::AF => self.registers.af(),
                };
                eprintln!("  PUSH {:?} ({:#04x})", register, value);
                self.push(value);
                self.pc.wrapping_add(1)
            }
            Instruction::Pop(register) => {
                let value = self.pop();
                eprintln!("  POP {:?} {:#04x}", register, value);
                match register {
                    Register16Alt::BC => self.registers.set_bc(value),
                    Register16Alt::DE => self.registers.set_de(value),
                    Register16Alt::HL => self.registers.set_hl(value),
                    Register16Alt::AF => self.registers.set_af(value),
                }
                self.pc.wrapping_add(1)
            }
            Instruction::Rot(rot, register) => match rot {
                Rot::Rl => {
                    let value = self.match_register(register);
                    eprintln!("  RL {:?}", register);
                    let new_value = self.rotate_left_through_carry(value, true);
                    self.write_register(register, new_value);

                    self.pc.wrapping_add(2)
                }
                _ => todo!("unimplemented instruction: {:?}", instruction),
            },
            Instruction::Rla => {
                eprintln!("  RLA");
                self.registers.a = self.rotate_left_through_carry(self.registers.a, false);
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

    fn match_register16(&self, register: Register16) -> u16 {
        match register {
            Register16::SP => self.sp,
            Register16::BC => self.registers.bc(),
            Register16::DE => self.registers.de(),
            Register16::HL => self.registers.hl(),
        }
    }

    fn write_register16(&mut self, register: Register16, value: u16) {
        match register {
            Register16::SP => self.sp = value,
            Register16::BC => self.registers.set_bc(value),
            Register16::DE => self.registers.set_de(value),
            Register16::HL => self.registers.set_hl(value),
        };
    }

    fn match_jump_condition(&self, condition: JumpTest) -> bool {
        match condition {
            JumpTest::NotZero => !self.registers.f.contains(Flags::Zero),
            JumpTest::Zero => self.registers.f.contains(Flags::Zero),
            JumpTest::NotCarry => !self.registers.f.contains(Flags::Carry),
            JumpTest::Carry => self.registers.f.contains(Flags::Carry),
            JumpTest::Always => true,
        }
    }

    fn push(&mut self, value: u16) {
        self.sp = self.sp.wrapping_sub(1);
        self.bus.write_byte(self.sp, ((value & 0xFF00) >> 8) as u8);

        self.sp = self.sp.wrapping_sub(1);
        self.bus.write_byte(self.sp, (value & 0xFF) as u8);
    }

    fn pop(&mut self) -> u16 {
        // BUG: If the stack pointer would wrap in the middle of this read, i think this will have
        // incorrect behaviour
        let word = self.bus.read_word(self.sp);
        self.sp = self.sp.wrapping_add(2);
        word
    }

    fn call(&mut self, should_jump: bool, address: u16) -> u16 {
        let next_pc = self.pc.wrapping_add(3);
        if should_jump {
            self.push(next_pc);
            address
        } else {
            next_pc
        }
    }

    fn retn(&mut self, should_jump: bool) -> u16 {
        if should_jump {
            self.pop()
        } else {
            self.pc.wrapping_add(1)
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
        self.debug_context.push(format!("A' = {:02x}", new_value));
        let flags = &mut self.registers.f;
        let zero = new_value == 0;
        flags.set(Flags::Zero, zero);
        self.debug_context.push(format!("Z = {}", zero as u8));
        flags.remove(make_bitflags!(Flags::{Subtraction | Carry | HalfCarry}));
        new_value
    }

    fn cp(&mut self, value: u8) {
        let flags = &mut self.registers.f;
        flags.set(Flags::Zero, self.registers.a == value);
        flags.insert(Flags::Subtraction);
        flags.set(
            Flags::HalfCarry,
            (self.registers.a & 0b1111) < (value & 0b1111),
        );
        flags.set(Flags::Carry, self.registers.a < value);
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
        let new_value = value.wrapping_add(1);
        let flags = &mut self.registers.f;
        flags.set(Flags::Zero, new_value == 0);
        flags.remove(Flags::Subtraction);
        // HalfCarry is set if the lower 4 bits added together don't fit in the lower 4 bits
        flags.set(Flags::HalfCarry, (value & 0b1111) + 1 > 0b1111);
        new_value
    }

    fn dec(&mut self, value: u8) -> u8 {
        let new_value = value.wrapping_sub(1);
        let flags = &mut self.registers.f;
        flags.set(Flags::Zero, new_value == 0);
        flags.insert(Flags::Subtraction);
        // HalfCarry is set if the lower 4 bits are 0, meaning we needed a bit from the upper 4 bits
        flags.set(Flags::HalfCarry, (value & 0b1111) == 0);
        new_value
    }

    fn rotate_left_through_carry(&mut self, value: u8, set_zero: bool) -> u8 {
        let carry = self.registers.f.contains(Flags::Carry) as u8;
        let new_value = value << 1 | carry;
        let flags = &mut self.registers.f;
        flags.set(Flags::Zero, new_value == 0 && set_zero);
        flags.remove(make_bitflags!(Flags::{Subtraction | HalfCarry}));
        flags.set(Flags::Carry, value >> 7 == 1);

        new_value
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_boot_rom() {
        let boot_rom = include_bytes!("../dmg_boot.bin");
        let test_rom = include_bytes!("../test_roms/cpu_instrs/individual/01-special.gb");
        let mut cpu = Cpu::default();
        cpu.bus.slice_mut()[0..256].copy_from_slice(boot_rom);
        cpu.bus.slice_mut()[256..32768].copy_from_slice(&test_rom[256..]);
        loop {
            cpu.step();
        }
    }
}
