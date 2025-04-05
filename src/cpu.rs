#![allow(dead_code)]

use enumflags2::make_bitflags;
use memorybus::MemoryBus;
use registers::{Flags, Registers};
use std::fmt::Write as _;
use structdiff::{Difference, StructDiff};
use tracing::trace;

use crate::disassembler::{
    instruction::{
        Alu, COrImmediate, Direction, HLOrImmediate, Instruction, JumpTest, LoadIndirect, LoadType,
        Register, Register16, Register16Alt, RegisterOrImmediate, Rot,
    },
    parse_instruction,
};

pub mod memorybus;
pub mod registers;

#[derive(Debug, PartialEq, Eq, Clone, Difference)]
pub struct Cpu {
    #[difference(recurse)]
    pub registers: Registers,
    /// The Program Counter register
    pub pc: u16,
    pub sp: u16,
    #[difference(recurse)]
    pub bus: MemoryBus,
    pub interrupts_enabled: bool,

    #[difference(skip)]
    debug_bytes_consumed: Vec<u8>,
    // Optionally used
    #[difference(skip)]
    debug_context: Vec<String>,
}

macro_rules! debug_context {
    ($self:ident, insert at $index:literal, $($tt:tt)*) => {
        #[cfg(debug_assertions)]
        $self.debug_context.insert($index, format!($($tt)*));
    };
    ($self:ident, $($tt:tt)*) => {
        #[cfg(debug_assertions)]
        $self.push_debug_context(format!($($tt)*));
    };
}

macro_rules! print_debug {
    ($self:ident, [$($tt:tt)*], [$($ctx:tt)*] $(,)?) => {
        #[cfg(debug_assertions)]
        $self.print_debug(&format!($($tt)*), &format!($($ctx)*));
    };

    ($self:ident, $($tt:tt)*) => {
        #[cfg(debug_assertions)]
        $self.print_debug(&format!($($tt)*), &$self.format_context());
    };
}

impl Cpu {
    pub fn new(boot_rom: Option<&[u8; 256]>, game_rom: &[u8], test_mode: bool) -> Self {
        // FIXME: support running without boot_rom
        // this will need us to set the registers to a good state
        match boot_rom {
            Some(_) => Self {
                registers: Registers::default(),
                pc: 0,
                sp: 0,
                bus: MemoryBus::new(boot_rom, game_rom, test_mode),
                interrupts_enabled: false,
                debug_bytes_consumed: Vec::default(),
                debug_context: Vec::default(),
            },
            None => Self {
                registers: Registers {
                    a: 0x01,
                    b: 0x00,
                    c: 0x13,
                    d: 0x00,
                    e: 0xD8,
                    h: 0x01,
                    l: 0x4D,
                    f: make_bitflags!(Flags::{Carry | HalfCarry | Zero}),
                },
                pc: 0x100,
                sp: 0xFFFE,
                bus: MemoryBus::new(boot_rom, game_rom, test_mode),
                interrupts_enabled: false,
                debug_bytes_consumed: Vec::default(),
                debug_context: Vec::default(),
            },
        }
    }

    pub fn step(&mut self) -> u8 {
        self.debug_context.clear();
        let slice = self.bus.slice_from(self.pc);
        let (after, instruction) = parse_instruction(&slice).unwrap();
        let bytes_consumed_len = slice.len() - after.len();
        self.debug_bytes_consumed
            .splice(.., slice[..bytes_consumed_len].iter().copied());
        let (next_pc, cycles) = self.execute(instruction);
        // eprintln!("{}", self.format_state()); // TODO: Log to a file instead

        self.bus.gpu.step(cycles);
        self.pc = next_pc;
        cycles
    }

    pub fn format_state(&self) -> String {
        format!(
            "A:{:02X} F:{:02X} B:{:02X} C:{:02X} D:{:02X} E:{:02X} H:{:02X} L:{:02X} SP:{:04X} PC:{:04X} PCMEM:{:02X},{:02X},{:02X},{:02X}\n",
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

    #[cfg(debug_assertions)]
    fn format_context(&self) -> String {
        self.debug_context
            .iter()
            .filter(|s| !s.is_empty())
            .cloned()
            .collect::<Vec<_>>()
            .join(", ")
    }

    #[cfg(debug_assertions)]
    fn print_debug(&self, opcode: &str, context: &str) {
        let bytes: String =
            self.debug_bytes_consumed
                .iter()
                .fold(String::new(), |mut output, byte| {
                    let _ = write!(output, "{byte:02X} ");
                    output
                });
        trace!("{:04X} {bytes:12} {opcode:32} ; {context}", self.pc);
    }
    #[cfg(debug_assertions)]
    fn push_debug_context(&mut self, ctx: String) {
        self.debug_context.push(ctx);
    }

    #[allow(clippy::too_many_lines, clippy::cognitive_complexity)]
    fn execute(&mut self, instruction: Instruction) -> (u16, u8) {
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
                        debug_context!(self, "{indirect_type} = {address:04X}");
                    }

                    match direction {
                        Direction::IntoA => {
                            let value = self.bus.read_byte(address);
                            debug_context!(self, "A' = {value:02X}");
                            print_debug!(self, "LD A, ({})", indirect_type.to_opcode_string());
                            self.registers.a = value;
                        }
                        Direction::FromA => {
                            let value = self.registers.a;
                            debug_context!(self, "A = {value:02X}");
                            print_debug!(self, "LD ({}), A", indirect_type.to_opcode_string());
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
                        LoadIndirect::Immediate(_) => (self.pc.wrapping_add(3), 16),
                        _ => (self.pc.wrapping_add(1), 8),
                    }
                }
                LoadType::Byte(register, source) => {
                    let value = match source {
                        RegisterOrImmediate::Immediate(x) => x,
                        RegisterOrImmediate::Register(reg) => {
                            let value = self.match_register(reg);
                            debug_context!(self, "{reg} = {value:02X}");
                            value
                        }
                    };
                    print_debug!(self, "LD {register}, {source}");
                    self.write_register(register, value);
                    match source {
                        RegisterOrImmediate::Immediate(_) => (self.pc.wrapping_add(2), 8),
                        RegisterOrImmediate::Register(_) => (self.pc.wrapping_add(1), 4),
                    }
                }
                LoadType::Word(register, source) => {
                    let source_value = match source {
                        HLOrImmediate::Immediate(x) => x,
                        HLOrImmediate::HL => self.registers.hl(),
                    };
                    self.write_register16(register, source_value);
                    print_debug!(self, "LD {register}, {source_value:02X}");
                    match source {
                        HLOrImmediate::Immediate(_) => (self.pc.wrapping_add(3), 12),
                        HLOrImmediate::HL => (self.pc.wrapping_add(1), 8),
                    }
                }
                LoadType::LastByteAddress(source, direction) => {
                    let offset = match source {
                        COrImmediate::C => {
                            debug_context!(self, "C = {:02X}", self.registers.c);
                            self.registers.c
                        }
                        COrImmediate::Immediate(x) => x,
                    };
                    let address = 0xFF00 + u16::from(offset);

                    match direction {
                        Direction::FromA => {
                            debug_context!(self, "A = {:02X}", self.registers.a);
                            print_debug!(self, "LDH ({source}), A");
                            self.bus.write_byte(address, self.registers.a);
                        }
                        Direction::IntoA => {
                            let value = self.bus.read_byte(address);
                            debug_context!(self, "({address:04X}) = {value:02X}");
                            print_debug!(self, "LDH A, ({source})");
                            self.registers.a = self.bus.read_byte(address);
                        }
                    }

                    match source {
                        COrImmediate::Immediate(_) => (self.pc.wrapping_add(2), 12),
                        COrImmediate::C => (self.pc.wrapping_add(1), 8),
                    }
                }
                LoadType::FromSp(target, offset) => match target {
                    HLOrImmediate::Immediate(address) => {
                        debug_context!(self, "SP = {}", self.sp);
                        self.bus.write_word(address, self.sp);
                        print_debug!(self, "LD ({address:04X}), SP");
                        (self.pc.wrapping_add(3), 20)
                    }
                    HLOrImmediate::HL => {
                        debug_context!(self, "SP = {}", self.sp);
                        let value = self.add_signed(self.sp, offset);
                        self.registers.set_hl(value);
                        debug_context!(self, insert at 1, "HL' = {value:04X}");
                        print_debug!(self, "LD HL, SP {offset:+}");
                        (self.pc.wrapping_add(2), 12)
                    }
                },
            },
            Instruction::AddHl(register) => {
                let value = self.match_register16(register);
                debug_context!(self, "HL = {:04X}", self.registers.hl());
                let new_value = self.add_hl(value);
                self.registers.set_hl(new_value);
                debug_context!(self, insert at 1, "HL' = {:04X}", self.registers.hl());
                print_debug!(self, "ADD HL, {register}");
                (self.pc.wrapping_add(1), 8)
            }
            Instruction::AddSp(offset) => {
                let value = self.sp;
                debug_context!(self, "SP = {:04X}", self.sp);
                let new_value = self.add_signed(value, offset);
                self.sp = new_value;
                debug_context!(self, insert at 1, "SP' = {:04X}", self.sp);
                print_debug!(self, "ADD SP, {offset}");
                (self.pc.wrapping_add(2), 16)
            }
            Instruction::Arithmetic(alu, source) => match alu {
                Alu::Add | Alu::Adc => {
                    let value = match source {
                        RegisterOrImmediate::Register(register) => {
                            let value = self.match_register(register);
                            debug_context!(self, "{register} = {value:02X}");
                            value
                        }
                        RegisterOrImmediate::Immediate(value) => value,
                    };
                    debug_context!(self, insert at 0, "A = {:02X}", self.registers.a);
                    self.registers.a = self.add(value, alu == Alu::Adc);
                    debug_context!(self, insert at 1, "A' = {:02X}", self.registers.a);
                    print_debug!(self, "{alu} {source}");

                    match source {
                        RegisterOrImmediate::Immediate(_) => (self.pc.wrapping_add(2), 8),
                        RegisterOrImmediate::Register(Register::HLIndirect) => {
                            (self.pc.wrapping_add(1), 8)
                        }
                        RegisterOrImmediate::Register(_) => (self.pc.wrapping_add(1), 4),
                    }
                }
                Alu::Sub | Alu::Sbc => {
                    let value = match source {
                        RegisterOrImmediate::Register(register) => {
                            let value = self.match_register(register);
                            debug_context!(self, "{register} = {value:02X}");
                            value
                        }
                        RegisterOrImmediate::Immediate(value) => value,
                    };
                    debug_context!(self, insert at 0, "A = {:02X}", self.registers.a);
                    self.registers.a = self.sub(value, alu == Alu::Sbc);
                    debug_context!(self, insert at 1, "A' = {:02X}", self.registers.a);
                    print_debug!(self, "{alu} {source}");

                    match source {
                        RegisterOrImmediate::Immediate(_) => (self.pc.wrapping_add(2), 8),
                        RegisterOrImmediate::Register(Register::HLIndirect) => {
                            (self.pc.wrapping_add(1), 8)
                        }
                        RegisterOrImmediate::Register(_) => (self.pc.wrapping_add(1), 4),
                    }
                }
                Alu::And => {
                    let value = match source {
                        RegisterOrImmediate::Register(register) => {
                            let value = self.match_register(register);
                            if !matches!(register, Register::A) {
                                debug_context!(self, "{register} = {value:02X}");
                            }
                            value
                        }
                        RegisterOrImmediate::Immediate(value) => value,
                    };

                    debug_context!(self, "A = {:02X}", self.registers.a);
                    self.registers.a = self.and(value);
                    print_debug!(self, "AND {source}");
                    match source {
                        RegisterOrImmediate::Immediate(_) => (self.pc.wrapping_add(2), 8),
                        RegisterOrImmediate::Register(Register::HLIndirect) => {
                            (self.pc.wrapping_add(1), 8)
                        }
                        RegisterOrImmediate::Register(_) => (self.pc.wrapping_add(1), 4),
                    }
                }
                Alu::Or => {
                    let value = match source {
                        RegisterOrImmediate::Register(register) => {
                            let value = self.match_register(register);
                            if !matches!(register, Register::A) {
                                debug_context!(self, "{register} = {value:02X}");
                            }
                            value
                        }
                        RegisterOrImmediate::Immediate(value) => value,
                    };

                    debug_context!(self, "A = {:02X}", self.registers.a);
                    self.registers.a = self.or(value);
                    print_debug!(self, "OR {source}");
                    match source {
                        RegisterOrImmediate::Immediate(_) => (self.pc.wrapping_add(2), 8),
                        RegisterOrImmediate::Register(Register::HLIndirect) => {
                            (self.pc.wrapping_add(1), 8)
                        }
                        RegisterOrImmediate::Register(_) => (self.pc.wrapping_add(1), 4),
                    }
                }
                Alu::Xor => {
                    let value = match source {
                        RegisterOrImmediate::Register(register) => {
                            let value = self.match_register(register);
                            if !matches!(register, Register::A) {
                                debug_context!(self, "{register} = {value:02X}");
                            }
                            value
                        }
                        RegisterOrImmediate::Immediate(value) => value,
                    };
                    debug_context!(self, "A = {:02X}", self.registers.a);
                    self.registers.a = self.xor(value);
                    print_debug!(self, "XOR {source}");
                    match source {
                        RegisterOrImmediate::Immediate(_) => (self.pc.wrapping_add(2), 8),
                        RegisterOrImmediate::Register(Register::HLIndirect) => {
                            (self.pc.wrapping_add(1), 8)
                        }
                        RegisterOrImmediate::Register(_) => (self.pc.wrapping_add(1), 4),
                    }
                }
                Alu::Cp => {
                    let value = match source {
                        RegisterOrImmediate::Register(register) => {
                            let value = self.match_register(register);
                            debug_context!(self, "{register} = {value:02X}");
                            value
                        }
                        RegisterOrImmediate::Immediate(value) => value,
                    };
                    self.cp(value);
                    print_debug!(self, "CP {source}");

                    match source {
                        RegisterOrImmediate::Immediate(_) => (self.pc.wrapping_add(2), 8),
                        RegisterOrImmediate::Register(Register::HLIndirect) => {
                            (self.pc.wrapping_add(1), 8)
                        }
                        RegisterOrImmediate::Register(_) => (self.pc.wrapping_add(1), 4),
                    }
                }
            },
            Instruction::Bit(bit, source) => {
                let value = self.match_register(source);
                let mask = 1 << bit;
                debug_context!(self, "{source} = {value:02X}");

                self.bit(mask, value);

                print_debug!(self, "BIT {bit}, {source}");
                match source {
                    Register::HLIndirect => (self.pc.wrapping_add(2), 12),
                    _ => (self.pc.wrapping_add(2), 8),
                }
            }
            Instruction::JR(condition, relative) => {
                let should_jump = self.match_jump_condition(condition);

                let target_address = self
                    .pc
                    .wrapping_add(2)
                    .wrapping_add_signed(i16::from(relative));
                print_debug!(self, "JR {condition} {target_address:04X}");

                self.relative_jump(should_jump, relative)
            }
            Instruction::JP(condition, target) => {
                let should_jump = self.match_jump_condition(condition);
                let address = match target {
                    HLOrImmediate::HL => {
                        let address = self.registers.hl();
                        debug_context!(self, "HL = {address:04X}");
                        address
                    }
                    HLOrImmediate::Immediate(address) => address,
                };
                print_debug!(self, "JP {condition} {target}");
                match target {
                    // there is no conditional HL jump, only conditional immediate
                    HLOrImmediate::HL => (address, 4),
                    HLOrImmediate::Immediate(_) => self.jump(should_jump, address),
                }
            }
            Instruction::Inc(register) => {
                let value = self.match_register(register);
                debug_context!(self, "{register} = {value:02X}");
                let new_value = self.inc(value);
                debug_context!(self, insert at 1, "{register}' = {new_value:02X}");
                self.write_register(register, new_value);

                print_debug!(self, "INC {register}");
                match register {
                    Register::HLIndirect => (self.pc.wrapping_add(1), 12),
                    _ => (self.pc.wrapping_add(1), 4),
                }
            }
            Instruction::Inc16(register) => {
                let value = self.match_register16(register);
                let new_value = value.wrapping_add(1);
                self.write_register16(register, new_value);
                print_debug!(
                    self,
                    ["INC {register}"],
                    ["{register} = {value:02X}, {register}' = {new_value:02X}"]
                );
                (self.pc.wrapping_add(1), 8)
            }
            Instruction::Dec(register) => {
                let value = self.match_register(register);
                let new_value = self.dec(value);
                self.write_register(register, new_value);

                debug_context!(
                    self, insert at 0, "{register} = {value:02X}, {register}' = {new_value:02X}",
                );
                print_debug!(self, "DEC {register}");
                match register {
                    Register::HLIndirect => (self.pc.wrapping_add(1), 12),
                    _ => (self.pc.wrapping_add(1), 4),
                }
            }
            Instruction::Dec16(register) => {
                let value = self.match_register16(register);
                let new_value = value.wrapping_sub(1);
                self.write_register16(register, new_value);
                print_debug!(
                    self,
                    ["DEC {register}"],
                    ["{register} = {value:04X}, {register}' = {new_value:04X}"]
                );
                (self.pc.wrapping_add(1), 8)
            }
            Instruction::Call(condition, address) => {
                let should_jump = self.match_jump_condition(condition);
                let pc = self.call(should_jump, address);

                print_debug!(self, "CALL {condition} {address:04X}");
                pc
            }
            Instruction::Ret(condition) => {
                let should_return = self.match_jump_condition(condition);
                let address = self.bus.read_word(self.sp);
                debug_context!(self, "(SP) = {address:04X}");
                let pc = self.retn(should_return);
                print_debug!(self, "RET {condition}");
                match condition {
                    JumpTest::Always => (pc.0, 16),
                    _ => pc,
                }
            }
            Instruction::Push(register) => {
                let value = match register {
                    Register16Alt::BC => self.registers.bc(),
                    Register16Alt::DE => self.registers.de(),
                    Register16Alt::HL => self.registers.hl(),
                    Register16Alt::AF => self.registers.af(),
                };
                debug_context!(self, "{register} = {value:04X}");
                self.push(value);
                print_debug!(self, "PUSH {register}");
                (self.pc.wrapping_add(1), 16)
            }
            Instruction::Pop(register) => {
                let value = self.pop();
                debug_context!(self, insert at 0, "{register}' = {value:04X}");
                print_debug!(self, "POP {register}");
                match register {
                    Register16Alt::BC => self.registers.set_bc(value),
                    Register16Alt::DE => self.registers.set_de(value),
                    Register16Alt::HL => self.registers.set_hl(value),
                    Register16Alt::AF => self.registers.set_af(value),
                }
                (self.pc.wrapping_add(1), 12)
            }
            Instruction::Rot(rot, register) => match rot {
                Rot::Rl => {
                    let value = self.match_register(register);
                    debug_context!(self, "{register} = {value:02X}");
                    let new_value = self.rotate_left_through_carry(value, true);
                    self.write_register(register, new_value);
                    debug_context!(self, insert at 1, "{register}' = {new_value:02X}");
                    print_debug!(self, "RL {register}");

                    match register {
                        Register::HLIndirect => (self.pc.wrapping_add(2), 16),
                        _ => (self.pc.wrapping_add(2), 8),
                    }
                }
                Rot::Rr => {
                    let value = self.match_register(register);
                    debug_context!(self, "{register} = {value:02X}");
                    let new_value = self.rotate_right_through_carry(value, true);
                    self.write_register(register, new_value);
                    debug_context!(self, insert at 1, "{register}' = {new_value:02X}");
                    print_debug!(self, "RR {register}");

                    match register {
                        Register::HLIndirect => (self.pc.wrapping_add(2), 16),
                        _ => (self.pc.wrapping_add(2), 8),
                    }
                }
                Rot::Srl | Rot::Sra => {
                    let value = self.match_register(register);
                    debug_context!(self, "{register} = {value:02X}");
                    let new_value = self.shift_right(value, rot == Rot::Sra);
                    self.write_register(register, new_value);
                    debug_context!(self, insert at 1, "{register}' = {new_value:02X}");
                    print_debug!(self, "{rot} {register}");

                    match register {
                        Register::HLIndirect => (self.pc.wrapping_add(2), 16),
                        _ => (self.pc.wrapping_add(2), 8),
                    }
                }
                _ => todo!("unimplemented instruction: {:?}", instruction),
            },
            Instruction::Rla => {
                debug_context!(self, "A = {:02X}", self.registers.a);
                self.registers.a = self.rotate_left_through_carry(self.registers.a, false);
                debug_context!(self, insert at 1, "A' = {:02X}", self.registers.a);

                print_debug!(self, "RLA");
                (self.pc.wrapping_add(1), 4)
            }
            Instruction::Rra => {
                debug_context!(self, "A = {:02X}", self.registers.a);
                self.registers.a = self.rotate_right_through_carry(self.registers.a, false);
                debug_context!(self, insert at 1, "A' = {:02X}", self.registers.a);

                print_debug!(self, "RRA");
                (self.pc.wrapping_add(1), 4)
            }
            Instruction::Nop => {
                print_debug!(self, "NOP");
                (self.pc.wrapping_add(1), 4)
            }
            Instruction::Di => {
                print_debug!(self, "DI");
                self.interrupts_enabled = false;
                (self.pc.wrapping_add(1), 4)
            }
            Instruction::Ei => {
                print_debug!(self, "EI");
                // FIXME: enabled after the next machine cycle?
                self.interrupts_enabled = true;
                (self.pc.wrapping_add(1), 4)
            }
            Instruction::Daa => {
                debug_context!(self, "A = {}", self.registers.a);
                self.registers.a = self.daa();
                debug_context!(self, insert at 1, "A' = {}", self.registers.a);
                print_debug!(self, "DAA");
                (self.pc.wrapping_add(1), 4)
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
        debug_context!(self, "{} = {}", flag, u8::from(contains));
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
        debug_context!(self, "SP = {:04X}", self.sp);
        self.sp = self.sp.wrapping_sub(1);
        self.bus.write_byte(self.sp, ((value & 0xFF00) >> 8) as u8);

        self.sp = self.sp.wrapping_sub(1);
        self.bus.write_byte(self.sp, (value & 0xFF) as u8);
        debug_context!(self, "SP' = {:04X}", self.sp);
    }

    fn pop(&mut self) -> u16 {
        // BUG: If the stack pointer would wrap in the middle of this read, i think this will have
        // incorrect behaviour
        let word = self.bus.read_word(self.sp);
        debug_context!(self, "SP = {:04X}", self.sp);
        self.sp = self.sp.wrapping_add(2);
        debug_context!(self, "SP' = {:04X}", self.sp);
        word
    }

    fn call(&mut self, should_jump: bool, address: u16) -> (u16, u8) {
        let next_pc = self.pc.wrapping_add(3);
        if should_jump {
            self.push(next_pc);
            (address, 24)
        } else {
            (next_pc, 12)
        }
    }

    fn retn(&mut self, should_jump: bool) -> (u16, u8) {
        if should_jump {
            (self.pop(), 20)
        } else {
            (self.pc.wrapping_add(1), 8)
        }
    }

    fn add(&mut self, value: u8, add_carry: bool) -> u8 {
        let carry = u8::from(add_carry && self.registers.f.contains(Flags::Carry));
        if add_carry {
            debug_context!(self, "C = {carry}");
        }
        let (new_value, overflow) = self.registers.a.overflowing_add(value);
        let (new_value, overflow2) = new_value.overflowing_add(carry);
        self.set_flag(Flags::Zero, new_value == 0);
        self.registers.f.remove(Flags::Subtraction);
        self.set_flag(Flags::Carry, overflow || overflow2);
        // HalfCarry is set if the lower 4 bits added together don't fit in the lower 4 bits
        self.set_flag(
            Flags::HalfCarry,
            (self.registers.a & 0b1111) + (value & 0b1111) + carry > 0b1111,
        );
        new_value
    }

    fn add_hl(&mut self, value: u16) -> u16 {
        let hl = self.registers.hl();
        let (new_value, overflow) = hl.overflowing_add(value);

        self.set_flag(Flags::Carry, overflow);
        let mask = 0b1111_1111_1111;
        self.set_flag(Flags::HalfCarry, (hl & mask) + (value & mask) > mask);
        self.registers.f.remove(Flags::Subtraction);

        new_value
    }

    fn add_signed(&mut self, value: u16, offset: i8) -> u16 {
        let offset = i16::from(offset);
        let new_value = value.wrapping_add_signed(offset);
        let mask = 0b1111;
        let mask_signed = 0b1111;
        self.set_flag(
            Flags::HalfCarry,
            (value & mask).wrapping_add_signed(offset & mask_signed) > mask,
        );
        // annoyingly, carry and half carry are set as if it was 8-bit, not 16-bit
        let mask = 0b1111_1111;
        let mask_signed = 0b1111_1111;
        self.set_flag(
            Flags::Carry,
            (value & mask).wrapping_add_signed(offset & mask_signed) > mask,
        );
        self.registers.f.remove(Flags::Subtraction | Flags::Zero);

        new_value
    }

    fn and(&mut self, value: u8) -> u8 {
        let new_value = self.registers.a & value;
        debug_context!(self, "A' = {new_value:02X}");
        self.set_flag(Flags::Zero, new_value == 0);
        self.registers
            .f
            .remove(make_bitflags!(Flags::{Subtraction | Carry}));
        self.set_flag(Flags::HalfCarry, true);
        new_value
    }

    fn or(&mut self, value: u8) -> u8 {
        let new_value = self.registers.a | value;
        debug_context!(self, "A' = {new_value:02X}");
        self.set_flag(Flags::Zero, new_value == 0);
        self.registers
            .f
            .remove(make_bitflags!(Flags::{Subtraction | Carry | HalfCarry}));
        new_value
    }

    fn xor(&mut self, value: u8) -> u8 {
        let new_value = self.registers.a ^ value;
        debug_context!(self, "A' = {new_value:02X}");
        self.set_flag(Flags::Zero, new_value == 0);
        self.registers
            .f
            .remove(make_bitflags!(Flags::{Subtraction | Carry | HalfCarry}));
        new_value
    }

    fn sub(&mut self, value: u8, sub_carry: bool) -> u8 {
        let carry = u8::from(sub_carry && self.registers.f.contains(Flags::Carry));
        if sub_carry {
            debug_context!(self, "C = {carry}");
        }
        let (new_value, overflow) = self.registers.a.overflowing_sub(value);
        let (new_value, overflow2) = new_value.overflowing_sub(carry);

        self.set_flag(Flags::Zero, new_value == 0);
        self.registers.f.insert(Flags::Subtraction);
        self.set_flag(Flags::Carry, overflow || overflow2);
        self.set_flag(
            Flags::HalfCarry,
            (self.registers.a & 0b1111) < (value & 0b1111) + carry,
        );
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

    const fn relative_jump(&self, should_jump: bool, offset: i8) -> (u16, u8) {
        let pc = self.pc.wrapping_add(2);
        if should_jump {
            (pc.wrapping_add_signed(offset as i16), 12)
        } else {
            (pc, 8)
        }
    }

    const fn jump(&self, should_jump: bool, address: u16) -> (u16, u8) {
        let pc = self.pc.wrapping_add(3);
        if should_jump { (address, 16) } else { (pc, 12) }
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

        debug_context!(self, "C = {carry}");
        self.set_flag(Flags::Carry, value >> 7 == 1);

        new_value
    }

    fn rotate_right_through_carry(&mut self, value: u8, set_zero: bool) -> u8 {
        let carry = u8::from(self.registers.f.contains(Flags::Carry));
        let new_value = value >> 1 | carry << 7;

        self.set_flag(Flags::Zero, new_value == 0 && set_zero);

        self.registers
            .f
            .remove(make_bitflags!(Flags::{Subtraction | HalfCarry}));

        debug_context!(self, "C = {carry}");
        self.set_flag(Flags::Carry, value & 1 == 1);

        new_value
    }

    fn shift_right(&mut self, value: u8, preserve_msb: bool) -> u8 {
        let mask = if preserve_msb { value & 0b1000_0000 } else { 0 };
        let new_value = (value >> 1) | mask;

        self.set_flag(Flags::Zero, new_value == 0);
        // set carry if we shifted a bit off
        self.set_flag(Flags::Carry, value & 1 == 1);
        self.registers
            .f
            .remove(make_bitflags!(Flags::{Subtraction | HalfCarry}));

        new_value
    }

    fn daa(&mut self) -> u8 {
        // thanks to <https://ehaskins.com/2018-01-30%20Z80%20DAA/>
        let value = self.registers.a;
        let half_carry = self.registers.f.contains(Flags::HalfCarry);
        let carry = self.registers.f.contains(Flags::Carry);
        let subtraction = self.registers.f.contains(Flags::Subtraction);

        // adjust first digit
        let adjust = if half_carry || (!subtraction && value & 0xF > 0x9) {
            0x6
        } else {
            0
        };
        // adjust second digit
        let adjust = if carry || (!subtraction && value > 0x99) {
            adjust + 0x60
        } else {
            adjust
        };

        let new_value = if subtraction {
            value.wrapping_sub(adjust)
        } else {
            value.wrapping_add(adjust)
        };

        self.set_flag(Flags::Zero, new_value == 0);
        self.registers
            .set_flag(Flags::Carry, carry || (!subtraction && value > 0x99));
        self.registers.f.remove(Flags::HalfCarry);

        new_value
    }

    fn set_flag(&mut self, flag: Flags, cond: bool) {
        let ctx = self.registers.set_flag(flag, cond);
        #[cfg(debug_assertions)]
        self.push_debug_context(ctx);
    }
}

#[cfg(test)]
mod test {
    use enumflags2::BitFlag;
    use jane_eyre::eyre;
    use serde::Deserialize;
    use serde_json::Value;
    use tracing::error;

    use super::*;
    // use pretty_assertions::assert_eq;

    #[test]
    fn test_boot_rom() {
        let boot_rom = include_bytes!("../dmg_boot.bin");
        let test_rom = include_bytes!("../test_roms/cpu_instrs/individual/01-special.gb");
        let mut cpu = Cpu::new(Some(boot_rom), test_rom, false);
        while cpu.pc < 0x100 {
            cpu.step();
        }
        // assert_eq!(cpu, Cpu::new(None, test_rom, false));
        // FIXME: print the diff cleaner
    }

    #[derive(Debug, Deserialize)]
    struct RamState {
        address: u16,
        value: u8,
    }

    #[derive(Debug, Deserialize)]
    struct GameboyState {
        pc: u16,
        sp: u16,
        a: u8,
        b: u8,
        c: u8,
        d: u8,
        e: u8,
        f: u8,
        h: u8,
        l: u8,
        ram: Vec<RamState>,
    }

    #[derive(Debug, Deserialize)]
    struct InstructionTest {
        name: String,
        initial: GameboyState,
        r#final: GameboyState,
        cycles: Vec<Value>,
    }

    #[test]
    fn test_single_step() -> eyre::Result<()> {
        let json = include_bytes!("../sm83/v1/0a.json");
        let tests = serde_json::from_slice::<Vec<InstructionTest>>(json)?;
        let results = tests
            .iter()
            .map(|test| {
                std::panic::catch_unwind(|| {
                    let mut initial = mock_cpu(&test.initial);
                    let after = mock_cpu(&test.r#final);
                    test.cycles.iter().skip(1).for_each(|_| {
                        initial.step();
                    });
                    let diffs = initial.diff_ref(&after);
                    if !diffs.is_empty() {
                        error!("test {} failed with: {diffs:?}", test.name);
                    }
                    assert_eq!(initial.pc, after.pc);
                    assert_eq!(diffs.len(), 0);
                })
            })
            .collect::<Vec<_>>();
        // FIXME: Deal with panics nicer
        assert_eq!(results.iter().filter(|x| x.is_ok()).count(), 1000);

        Ok(())
    }

    fn mock_cpu(state: &GameboyState) -> Cpu {
        let mut initial = Cpu {
            registers: Registers {
                a: state.a,
                b: state.b,
                c: state.c,
                d: state.d,
                e: state.e,
                h: state.h,
                l: state.l,
                f: Flags::from_bits_truncate(state.f),
            },
            pc: state.pc,
            sp: state.sp,
            bus: MemoryBus::new(None, &[], false),
            interrupts_enabled: false,
            debug_bytes_consumed: Vec::new(),
            debug_context: Vec::new(),
        };
        for write in &state.ram {
            initial.bus.write_byte(write.address, write.value);
        }

        initial
    }
}
