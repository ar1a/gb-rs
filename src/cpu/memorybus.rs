use crate::gpu::{Gpu, VRAM_BEGIN, VRAM_END};

pub const IO_BEGIN: usize = 0xFF00;
pub const IO_END: usize = 0xFF7F;
pub const IO_SIZE: usize = IO_END - IO_BEGIN + 1;

pub const HRAM_BEGIN: usize = 0xFF80;
pub const HRAM_END: usize = 0xFFFE;
pub const HRAM_SIZE: usize = HRAM_END - HRAM_BEGIN + 1;

#[derive(Debug)]
pub(crate) struct MemoryBus {
    // FIXME: separate into memory segments
    memory: [u8; 0xFFFF],
    gpu: Gpu,

    // TODO: Implement io as a struct
    io: [u8; IO_SIZE],
}

impl Default for MemoryBus {
    fn default() -> Self {
        Self {
            memory: [0; 0xFFFF],
            gpu: Gpu::default(),

            io: [0; IO_SIZE],
        }
    }
}

impl MemoryBus {
    pub fn read_byte(&self, address: u16) -> u8 {
        let address = address as usize;
        match address {
            00..=0x3FFF => self.memory[address],
            VRAM_BEGIN..=VRAM_END => self.gpu.read_vram(address - VRAM_BEGIN),
            IO_BEGIN..=IO_END => self.io[address - IO_BEGIN],
            HRAM_BEGIN..HRAM_END => self.memory[address],
            _ => todo!("memory region not mapped yet: {:#4x}", address),
        }
    }
    pub fn write_byte(&mut self, address: u16, value: u8) {
        let address = address as usize;
        match address {
            00..=0x3FFF => panic!("attempted to write to ROM"),
            VRAM_BEGIN..=VRAM_END => self.gpu.write_vram(address - VRAM_BEGIN, value),
            IO_BEGIN..=IO_END => self.io[address - IO_BEGIN] = value,
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
