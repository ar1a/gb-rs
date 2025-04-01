#![allow(dead_code)]

use crate::gpu::tile::{Tile, empty_tile};

pub const VRAM_BEGIN: usize = 0x8000;
pub const VRAM_END: usize = 0x9FFF;
pub const VRAM_SIZE: usize = VRAM_END - VRAM_BEGIN + 1;

mod tile;

#[derive(Debug)]
pub struct Gpu {
    vram: [u8; VRAM_SIZE],
    tile_set: [Tile; 384],
}

impl Default for Gpu {
    fn default() -> Self {
        Self {
            vram: [0; VRAM_SIZE],
            tile_set: [empty_tile(); 384],
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
