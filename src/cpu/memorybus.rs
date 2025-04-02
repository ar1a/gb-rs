use bitvec::array::BitArray;
use enumflags2::BitFlag;

use crate::gpu::{Gpu, LCDControl, VRAM_BEGIN, VRAM_END};

pub const IO_BEGIN: usize = 0xFF00;
pub const IO_END: usize = 0xFF7F;
pub const IO_SIZE: usize = IO_END - IO_BEGIN + 1;

pub const HRAM_BEGIN: usize = 0xFF80;
pub const HRAM_END: usize = 0xFFFE;
pub const HRAM_SIZE: usize = HRAM_END - HRAM_BEGIN + 1;

#[derive(Debug)]
pub struct MemoryBus {
    // FIXME: separate into memory segments
    memory: Box<[u8]>,
    pub gpu: Gpu,
}

impl Default for MemoryBus {
    fn default() -> Self {
        Self {
            memory: vec![0; 0xFFFF].into_boxed_slice(),
            gpu: Gpu::default(),
        }
    }
}

impl MemoryBus {
    pub fn read_byte(&self, address: u16) -> u8 {
        let address = address as usize;
        match address {
            00..=0x3FFF => self.memory[address],
            VRAM_BEGIN..=VRAM_END => self.gpu.read_vram(address - VRAM_BEGIN),
            IO_BEGIN..=IO_END => self.read_io_register(address),
            HRAM_BEGIN..HRAM_END => self.memory[address],
            _ => todo!("memory region not mapped yet: {:#4x}", address),
        }
    }
    pub fn write_byte(&mut self, address: u16, value: u8) {
        let address = address as usize;
        match address {
            00..=0x3FFF => panic!("attempted to write to ROM"),
            VRAM_BEGIN..=VRAM_END => self.gpu.write_vram(address - VRAM_BEGIN, value),
            IO_BEGIN..=IO_END => self.write_io_register(address, value),
            HRAM_BEGIN..HRAM_END => self.memory[address] = value,
            _ => todo!("memory region not mapped yet: {:#4x}", address),
        }
    }

    pub fn read_word(&self, address: u16) -> u16 {
        let bytes = [self.read_byte(address), self.read_byte(address + 1)];
        u16::from_le_bytes(bytes)
    }
    pub fn write_word(&mut self, address: u16, value: u16) {
        let bytes = u16::to_le_bytes(value);
        self.write_byte(address, bytes[0]);
        self.write_byte(address + 1, bytes[1]);
    }

    #[allow(clippy::match_same_arms)]
    fn write_io_register(&mut self, address: usize, value: u8) {
        match address {
            0xFF11 => { /* Sound Ch1 Length Timer and Duty Cycle */ }
            0xFF12 => { /* Sound Ch1 Volume and Envelope */ }
            0xFF13 => { /* Sound Ch1 Period Low */ }
            0xFF14 => { /* Sound Ch1 Period High and Control */ }
            0xFF24 => { /* Master Volume and VIN panning */ }
            0xFF25 => { /* Sound Panning */ }
            0xFF26 => { /* Sound Enabled */ }
            0xFF40 => self.gpu.lcd_control = LCDControl::from_bits(value).unwrap(),
            0xFF42 => self.gpu.viewport_y_offset = value,
            0xFF47 => self.gpu.background_colours = BitArray::new([value]),
            _ => todo!("implement io register write {address:04X}"),
        }
    }

    fn read_io_register(&self, address: usize) -> u8 {
        match address {
            0xFF40 => self.gpu.lcd_control.bits(),
            0xFF42 => self.gpu.viewport_y_offset,
            0xFF44 => self.gpu.line,
            _ => todo!("implement io register read {address:04X}"),
        }
    }

    // FIXME: memory map
    pub fn slice_from(&self, pc: u16) -> &[u8] {
        &self.memory[pc as usize..]
    }

    // FIXME: memory map
    pub fn slice(&self) -> &[u8] {
        &self.memory
    }

    // FIXME: memory map
    pub fn slice_mut(&mut self) -> &mut [u8] {
        &mut self.memory
    }
}
