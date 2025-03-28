#![allow(dead_code)]

use bitvec::{BitArr, array::BitArray, order::Msb0};
use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::FromPrimitive;
pub const VRAM_BEGIN: usize = 0x8000;
pub const VRAM_END: usize = 0x9FFF;
pub const VRAM_SIZE: usize = VRAM_END - VRAM_BEGIN + 1;

#[derive(Debug, Clone, Copy, FromPrimitive, ToPrimitive)]
enum Pixel {
    Zero = 0,
    One = 1,
    Two = 2,
    Three = 3,
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct TileRow {
    tiles: BitArr!(for 16, in u8, Msb0),
}

type Tile = [TileRow; 8];
const fn empty_tile() -> Tile {
    [TileRow {
        tiles: BitArray::ZERO,
    }; 8]
}

fn from_bytes_row(bytes: [u8; 2]) -> TileRow {
    let tiles = BitArray::new(bytes);
    TileRow { tiles }
}

fn from_bytes_tile(bytes: [u8; 16]) -> Tile {
    bytes
        .chunks_exact(2)
        .map(|bytes| from_bytes_row(bytes.try_into().unwrap()))
        .collect::<Vec<_>>()
        .try_into()
        .unwrap()
}

impl TileRow {
    fn get_tile(self, tile: u8) -> Pixel {
        let tile = usize::from(tile);
        let lsb = u8::from(self.tiles[tile]);
        let msb = u8::from(self.tiles[tile + 8]);
        Pixel::from_u8(msb << 1 | lsb).unwrap()
    }

    const fn iter(&self) -> TileRowIterator {
        TileRowIterator {
            tile_row: self,
            index: 0,
        }
    }
}

struct TileRowIterator<'a> {
    tile_row: &'a TileRow,
    index: u8,
}

impl Iterator for TileRowIterator<'_> {
    type Item = Pixel;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < 8 {
            let result = Some(self.tile_row.get_tile(self.index));
            self.index += 1;
            result
        } else {
            None
        }
    }
}

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
        let tile = from_bytes_tile(bytes);
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
