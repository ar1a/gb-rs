use crate::gpu::{Gpu, VRAM_BEGIN, VRAM_END};

pub const IO_BEGIN: usize = 0xFF00;
pub const IO_END: usize = 0xFF7F;
pub const IO_SIZE: usize = IO_END - IO_BEGIN + 1;

#[derive(Debug)]
pub(crate) struct MemoryBus {
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
            _ => todo!("memory region not mapped yet: {:#4x}", address),
        }
    }
    pub fn write_byte(&mut self, address: u16, value: u8) {
        let address = address as usize;
        match address {
            00..=0x3FFF => panic!("attempted to write to ROM"),
            VRAM_BEGIN..=VRAM_END => self.gpu.write_vram(address - VRAM_BEGIN, value),
            IO_BEGIN..=IO_END => self.io[address - IO_BEGIN] = value,
            _ => todo!("memory region not mapped yet: {:#4x}", address),
        }
    }

    pub fn read_word(&self, address: u16) -> u16 {
        let address = address as usize;
        u16::from_le_bytes(self.memory[address..=address + 1].try_into().unwrap())
    }
    pub fn write_word(&mut self, address: u16, value: u16) {
        let address = address as usize;
        let bytes = u16::to_le_bytes(value);
        self.memory[address..=address + 1].copy_from_slice(&bytes);
    }

    pub fn slice_from(&self, pc: u16) -> &[u8] {
        &self.memory[pc as usize..]
    }

    pub fn slice(&self) -> &[u8] {
        &self.memory
    }
    pub fn slice_mut(&mut self) -> &mut [u8] {
        &mut self.memory
    }
}
