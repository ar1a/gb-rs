#![allow(dead_code)]

use tracing::trace;

use crate::gpu::tile::{Tile, empty_tile};

pub const VRAM_BEGIN: usize = 0x8000;
pub const VRAM_END: usize = 0x9FFF;
pub const VRAM_SIZE: usize = VRAM_END - VRAM_BEGIN + 1;

pub const WIDTH: usize = 160;
pub const HEIGHT: usize = 144;

mod tile;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    OamScan = 2,
    Drawing = 3,
    HBlank = 0,
    VBlank = 1,
}

#[derive(Debug)]
pub struct Gpu {
    vram: [u8; VRAM_SIZE],
    tile_set: [Tile; 384],
    pub buffer: Box<[u8; WIDTH * HEIGHT * 3]>,
    cycles: u16,
    pub line: u8,
    pub mode: Mode,
}

impl Default for Gpu {
    fn default() -> Self {
        Self {
            vram: [0; VRAM_SIZE],
            tile_set: [empty_tile(); 384],
            buffer: vec![0; WIDTH * HEIGHT * 3]
                .into_boxed_slice()
                .try_into()
                .unwrap(),
            cycles: 0,
            line: 0,
            mode: Mode::HBlank,
        }
    }
}

impl Gpu {
    pub fn step(&mut self, cycles: u8) {
        self.cycles = self.cycles.wrapping_add(u16::from(cycles));
        match self.mode {
            Mode::OamScan => {
                if self.cycles >= 80 {
                    self.cycles %= 80;
                    self.mode = Mode::Drawing;
                }
            }
            Mode::Drawing => {
                if self.cycles >= 172 {
                    self.cycles %= 172;
                    self.mode = Mode::HBlank;
                    self.render_line();
                }
            }
            Mode::HBlank => {
                if self.cycles >= 204 {
                    self.cycles %= 204;
                    self.line += 1;
                    if self.line >= 144 {
                        self.mode = Mode::VBlank;
                    } else {
                        self.mode = Mode::OamScan;
                    }
                }
            }
            Mode::VBlank => {
                if self.cycles >= 456 {
                    self.cycles %= 456;
                }
                self.line += 1;

                if self.line >= 154 {
                    self.mode = Mode::OamScan;
                    self.line = 0;
                    // FIXME: only here for testing
                    self.buffer.fill(0);
                }
            }
        }
    }
    pub const fn read_vram(&self, index: usize) -> u8 {
        self.vram[index]
    }

    pub const fn write_vram(&mut self, index: usize, value: u8) {
        self.vram[index] = value;
    }

    fn render_line(&mut self) {
        let mut line_offset = self.line as usize * WIDTH * 3;
        for i in 0..WIDTH {
            self.buffer[line_offset + i] = 255;
            self.buffer[line_offset + i + 1] = 255;
            self.buffer[line_offset + i + 2] = 255;
            line_offset += 2;
        }
    }
}

#[cfg(test)]
mod test {
    use num_traits::ToPrimitive;
    use std::fmt::Write as _;

    use super::*;

    #[test]
    fn test_tilerow() {
        let bytes = [
            0xFF, 0x00, 0x7E, 0xFF, 0x85, 0x81, 0x89, 0x83, 0x93, 0x85, 0xA5, 0x8B, 0xC9, 0x97,
            0x7E, 0xFF,
        ];
        let tile = tile::from_bytes_tile(bytes);
        let mut output = String::with_capacity(64);
        for tilerow in &tile {
            tilerow.iter().for_each(|tile| {
                let _ = write!(output, "{}", tile.to_u8().unwrap());
            });
        }
        assert_eq!(
            output,
            "1111111123333332300001033000102330010213301021233102122323333332"
        );
    }
}
