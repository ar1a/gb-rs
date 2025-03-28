#![allow(dead_code)]
pub const VRAM_BEGIN: usize = 0x8000;
pub const VRAM_END: usize = 0x9FFF;
pub const VRAM_SIZE: usize = VRAM_END - VRAM_BEGIN + 1;

#[derive(Debug)]
pub struct Gpu {
    vram: [u8; VRAM_SIZE],
}

impl Default for Gpu {
    fn default() -> Self {
        Self {
            vram: [0; VRAM_SIZE],
        }
    }
}

impl Gpu {
    pub const fn read_vram(&self, index: usize) -> u8 {
        self.vram[index]
    }

    pub const fn write_vram(&mut self, index: usize, value: u8) {
        self.vram[index] = value;
    }
}
