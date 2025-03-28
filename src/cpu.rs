#![allow(dead_code)]

use enumflags2::make_bitflags;
use memorybus::MemoryBus;
use registers::{Flags, Registers};
use std::fmt::Write as _;

use crate::disassembler::{
    instruction::{
        Alu, COrImmediate, Direction, Instruction, JumpTest, LoadIndirect, LoadType,
        LoadWordSource, Register, Register16, Register16Alt, RegisterOrImmediate, Rot,
    },
    parse_instruction,
};

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
        eprint!("{:04X}", self.pc);
        let bytes: String =
            self.debug_bytes_consumed
                .iter()
                .fold(String::new(), |mut output, byte| {
                    let _ = write!(output, "{byte:02X} ");
                    output
                });
        eprint!(" {bytes:12}");
        eprint!("{opcode:32}");
        eprintln!(" ; {context}");
    }

    #[allow(clippy::too_many_lines)]
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
                        LoadIndirect::Immediate(address) => address,
                    };
                    if !matches!(indirect_type, LoadIndirect::Immediate(_)) {
                        self.debug_context
                            .push(format!("{indirect_type} = {address:04X}"));
                    }

                    match direction {
                        Direction::IntoA => {
                            let value = self.bus.read_byte(address);
                            self.debug_context.push(format!("A' = {value:02X}"));
                            self.print_debug(
                                &format!("LD A, ({})", indirect_type.to_opcode_string()),
                                &self.format_context(),
                            );
                            self.registers.a = value;
                        }
                        Direction::FromA => {
                            let value = self.registers.a;
                            self.debug_context.push(format!("A = {value:02X}"));
                            self.print_debug(
                                &format!("LD ({}), A", indirect_type.to_opcode_string()),
                                &self.format_context(),
                            );
                            self.bus.write_byte(address, value);
                        }
                    }

                    let adjust = match indirect_type {
                        LoadIndirect::HLDec => -1,
                        LoadIndirect::HLInc => 1,
                        _ => 0,
                    };
                    if adjust != 0 {
                        self.registers.set_hl(address.wrapping_add_signed(adjust));
                    }
                    match indirect_type {
                        LoadIndirect::Immediate(_) => self.pc.wrapping_add(3),
                        _ => self.pc.wrapping_add(1),
                    }
                }
                LoadType::Byte(register, source) => {
                    let value = match source {
                        RegisterOrImmediate::Immediate(x) => x,
                        RegisterOrImmediate::Register(reg) => {
                            let value = self.match_register(reg);
                            self.debug_context.push(format!("{reg} = {value:02X}"));
                            value
                        }
                    };
                    self.print_debug(&format!("LD {register}, {source}"), &self.format_context());
                    self.write_register(register, value);
                    match source {
                        RegisterOrImmediate::Immediate(_) => self.pc.wrapping_add(2),
                        RegisterOrImmediate::Register(_) => self.pc.wrapping_add(1),
                    }
                }
                LoadType::Word(register, source) => {
                    let source_value = match source {
                        LoadWordSource::Immediate(x) => x,
                    };
                    self.write_register16(register, source_value);
                    self.print_debug(&format!("LD {register}, {source_value:02X}"), "");
                    match source {
                        LoadWordSource::Immediate(_) => self.pc.wrapping_add(3),
                    }
                }
                LoadType::LastByteAddress(source, direction) => {
                    let offset = match source {
                        COrImmediate::C => {
                            self.debug_context
                                .push(format!("C = {:02X}", self.registers.c));
                            self.registers.c
                        }
                        COrImmediate::Immediate(x) => x,
                    };
                    let address = 0xFF00 + u16::from(offset);

                    match direction {
                        Direction::FromA => {
                            self.debug_context
                                .push(format!("A = {:02X}", self.registers.a));
                            self.print_debug(&format!("LDH ({source}), A"), &self.format_context());
                            self.bus.write_byte(address, self.registers.a);
                        }
                        Direction::IntoA => {
                            let value = self.bus.read_byte(address);
                            self.debug_context
                                .push(format!("({address:04X}) = {value:02X}"));
                            self.print_debug(&format!("LDH A, ({source})"), &self.format_context());
                            self.registers.a = self.bus.read_byte(address);
                        }
                    }

                    match source {
                        COrImmediate::Immediate(_) => self.pc.wrapping_add(2),
                        COrImmediate::C => self.pc.wrapping_add(1),
                    }
                }
            },
            Instruction::Arithmetic(alu, source) => match alu {
                Alu::Xor => {
                    let value = match source {
                        RegisterOrImmediate::Register(register) => {
                            let value = self.match_register(register);
                            if !matches!(register, Register::A) {
                                self.debug_context.push(format!("{register} = {value:02X}"));
                            }
                            value
                        }
                        RegisterOrImmediate::Immediate(value) => value,
                    };
                    self.debug_context
                        .push(format!("A = {:02X}", self.registers.a));
                    self.registers.a = self.xor(value);
                    self.print_debug(&format!("XOR {source}"), &self.format_context());
                    match source {
                        RegisterOrImmediate::Immediate(_) => self.pc.wrapping_add(2),
                        RegisterOrImmediate::Register(_) => self.pc.wrapping_add(1),
                    }
                }
                Alu::Cp => {
                    let value = match source {
                        RegisterOrImmediate::Register(register) => {
                            let value = self.match_register(register);
                            self.debug_context.push(format!("{register} = {value:02X}"));
                            value
                        }
                        RegisterOrImmediate::Immediate(value) => value,
                    };
                    self.cp(value);
                    self.print_debug(&format!("CP {source}"), &self.format_context());

                    match source {
                        RegisterOrImmediate::Immediate(_) => self.pc.wrapping_add(2),
                        RegisterOrImmediate::Register(_) => self.pc.wrapping_add(1),
                    }
                }
                _ => todo!("alu opertion: {:?} {:?}", alu, source),
            },
            Instruction::Bit(bit, source) => {
                let value = self.match_register(source);
                let mask = 1 << bit;
                self.debug_context.push(format!("{source} = {value:02X}"));

                self.bit(mask, value);

                self.print_debug(&format!("BIT {bit}, {source}"), &self.format_context());
                self.pc.wrapping_add(2)
            }
            Instruction::JR(condition, relative) => {
                let should_jump = self.match_jump_condition(condition);

                let target_address = self
                    .pc
                    .wrapping_add(2)
                    .wrapping_add_signed(i16::from(relative));
                self.print_debug(
                    &format!("JR {condition} {target_address:04X}"),
                    &self.format_context(),
                );

                self.relative_jump(should_jump, relative)
            }
            Instruction::Inc(register) => {
                let value = self.match_register(register);
                self.debug_context.push(format!("{register} = {value:02X}"));
                let new_value = self.inc(value);
                self.debug_context
                    .insert(1, format!("{register}' = {new_value:02X}"));
                self.write_register(register, new_value);

                self.print_debug(&format!("INC {register}"), &self.format_context());
                self.pc.wrapping_add(1)
            }
            Instruction::Inc16(register) => {
                let value = self.match_register16(register);
                let new_value = value.wrapping_add(1);
                self.write_register16(register, new_value);
                self.print_debug(
                    &format!("INC {register}"),
                    &format!("{register} = {value:02X}, {register}' = {new_value:02X}"),
                );
                self.pc.wrapping_add(1)
            }
            Instruction::Dec(register) => {
                let value = self.match_register(register);
                let new_value = self.dec(value);
                self.write_register(register, new_value);

                self.print_debug(
                    &format!("DEC {register}"),
                    &format!("{register} = {value:02X}, {register}' = {new_value:02X}"),
                );
                self.pc.wrapping_add(1)
            }
            Instruction::Dec16(register) => {
                let value = self.match_register16(register);
                let new_value = value.wrapping_sub(1);
                self.write_register16(register, new_value);
                self.print_debug(
                    &format!("DEC {register}"),
                    &format!("{register} = {value:04X}, {register}' = {new_value:04X}"),
                );
                self.pc.wrapping_add(1)
            }
            Instruction::Call(condition, address) => {
                let should_jump = self.match_jump_condition(condition);
                let pc = self.call(should_jump, address);

                self.print_debug(
                    &format!("CALL {condition} {address:04X}"),
                    &self.format_context(),
                );
                pc
            }
            Instruction::Ret => {
                let address = self.bus.read_word(self.sp);
                self.debug_context.push(format!("(SP) = {address:04X}"));
                let pc = self.retn(true);
                self.print_debug("RET", &self.format_context());
                pc
            }
            Instruction::Push(register) => {
                let value = match register {
                    Register16Alt::BC => self.registers.bc(),
                    Register16Alt::DE => self.registers.de(),
                    Register16Alt::HL => self.registers.hl(),
                    Register16Alt::AF => self.registers.af(),
                };
                self.debug_context.push(format!("{register} = {value:04X}"));
                self.push(value);
                self.print_debug(&format!("PUSH {register}"), &self.format_context());
                self.pc.wrapping_add(1)
            }
            Instruction::Pop(register) => {
                let value = self.pop();
                self.debug_context
                    .insert(0, format!("{register}' = {value:04X}"));
                self.print_debug(&format!("POP {register}"), &self.format_context());
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
                    self.debug_context.push(format!("{register} = {value:02X}"));
                    let new_value = self.rotate_left_through_carry(value, true);
                    self.write_register(register, new_value);
                    self.debug_context
                        .insert(1, format!("{register}' = {new_value:02X}"));
                    self.print_debug(&format!("RL {register}"), &self.format_context());

                    self.pc.wrapping_add(2)
                }
                _ => todo!("unimplemented instruction: {:?}", instruction),
            },
            Instruction::Rla => {
                self.debug_context
                    .push(format!("A = {:02X}", self.registers.a));
                self.registers.a = self.rotate_left_through_carry(self.registers.a, false);
                self.debug_context
                    .insert(1, format!("A' = {:02X}", self.registers.a));

                self.print_debug("RLA", &self.format_context());
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

    const fn match_register16(&self, register: Register16) -> u16 {
        match register {
            Register16::SP => self.sp,
            Register16::BC => self.registers.bc(),
            Register16::DE => self.registers.de(),
            Register16::HL => self.registers.hl(),
        }
    }

    const fn write_register16(&mut self, register: Register16, value: u16) {
        match register {
            Register16::SP => self.sp = value,
            Register16::BC => self.registers.set_bc(value),
            Register16::DE => self.registers.set_de(value),
            Register16::HL => self.registers.set_hl(value),
        }
    }

    fn flags_contains(&mut self, flag: Flags) -> bool {
        let contains = self.registers.f.contains(flag);
        self.debug_context
            .push(format!("{} = {}", flag, u8::from(contains)));
        contains
    }

    fn match_jump_condition(&mut self, condition: JumpTest) -> bool {
        match condition {
            JumpTest::NotZero => !self.flags_contains(Flags::Zero),
            JumpTest::Zero => self.flags_contains(Flags::Zero),
            JumpTest::NotCarry => !self.flags_contains(Flags::Carry),
            JumpTest::Carry => self.flags_contains(Flags::Carry),
            JumpTest::Always => true,
        }
    }

    fn push(&mut self, value: u16) {
        self.debug_context.push(format!("SP = {:04X}", self.sp));
        self.sp = self.sp.wrapping_sub(1);
        self.bus.write_byte(self.sp, ((value & 0xFF00) >> 8) as u8);

        self.sp = self.sp.wrapping_sub(1);
        self.bus.write_byte(self.sp, (value & 0xFF) as u8);
        self.debug_context.push(format!("SP' = {:04X}", self.sp));
    }

    fn pop(&mut self) -> u16 {
        // BUG: If the stack pointer would wrap in the middle of this read, i think this will have
        // incorrect behaviour
        let word = self.bus.read_word(self.sp);
        self.debug_context.push(format!("SP = {:04X}", self.sp));
        self.sp = self.sp.wrapping_add(2);
        self.debug_context.push(format!("SP' = {:04X}", self.sp));
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
        self.set_flag(Flags::Zero, new_value == 0);
        self.registers.f.remove(Flags::Subtraction);
        self.set_flag(Flags::Carry, overflow);
        // HalfCarry is set if the lower 4 bits added together don't fit in the lower 4 bits
        self.set_flag(
            Flags::HalfCarry,
            (self.registers.a & 0b1111) + (value & 0b1111) > 0b1111,
        );
        new_value
    }

    fn xor(&mut self, value: u8) -> u8 {
        let new_value = self.registers.a ^ value;
        self.debug_context.push(format!("A' = {new_value:02X}"));
        self.set_flag(Flags::Zero, new_value == 0);
        self.registers
            .f
            .remove(make_bitflags!(Flags::{Subtraction | Carry | HalfCarry}));
        new_value
    }

    fn cp(&mut self, value: u8) {
        self.set_flag(Flags::Zero, self.registers.a == value);

        self.registers.f.insert(Flags::Subtraction);

        self.set_flag(
            Flags::HalfCarry,
            (self.registers.a & 0b1111) < (value & 0b1111),
        );

        self.set_flag(Flags::Carry, self.registers.a < value);
    }

    fn bit(&mut self, mask: u8, value: u8) {
        self.set_flag(Flags::Zero, value & mask == 0);
        self.registers.f.remove(Flags::Subtraction);
        self.registers.f.insert(Flags::HalfCarry);
    }

    const fn relative_jump(&self, should_jump: bool, offset: i8) -> u16 {
        let pc = self.pc.wrapping_add(2);
        if should_jump {
            pc.wrapping_add_signed(offset as i16)
        } else {
            pc
        }
    }

    fn inc(&mut self, value: u8) -> u8 {
        let new_value = value.wrapping_add(1);
        self.set_flag(Flags::Zero, new_value == 0);
        self.registers.f.remove(Flags::Subtraction);
        // HalfCarry is set if the lower 4 bits added together don't fit in the lower 4 bits
        let half_carry = (value & 0b1111) + 1 > 0b1111;
        self.set_flag(Flags::HalfCarry, half_carry);
        new_value
    }

    fn dec(&mut self, value: u8) -> u8 {
        let new_value = value.wrapping_sub(1);
        self.set_flag(Flags::Zero, new_value == 0);
        self.debug_context
            .push(format!("Z' = {}", u8::from(new_value == 0)));
        self.registers.f.insert(Flags::Subtraction);
        // HalfCarry is set if the lower 4 bits are 0, meaning we needed a bit from the upper 4 bits
        self.set_flag(Flags::HalfCarry, value.trailing_zeros() >= 4);
        new_value
    }

    fn rotate_left_through_carry(&mut self, value: u8, set_zero: bool) -> u8 {
        let carry = u8::from(self.registers.f.contains(Flags::Carry));
        let new_value = value << 1 | carry;

        self.set_flag(Flags::Zero, new_value == 0 && set_zero);

        self.registers
            .f
            .remove(make_bitflags!(Flags::{Subtraction | HalfCarry}));

        self.debug_context.push(format!("C = {carry}"));
        self.set_flag(Flags::Carry, value >> 7 == 1);

        new_value
    }

    fn set_flag(&mut self, flag: Flags, cond: bool) {
        self.debug_context.push(self.registers.set_flag(flag, cond));
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
        while cpu.pc < 0x100 {
            cpu.step();
        }
    }
}
