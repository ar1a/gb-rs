use bitvec::{BitArr, array::BitArray, order::Msb0};
use num_derive::{FromPrimitive, ToPrimitive};
use num_traits::FromPrimitive;

#[derive(Debug, Clone, Copy, FromPrimitive, ToPrimitive)]
pub enum Pixel {
    Zero = 0,
    One = 1,
    Two = 2,
    Three = 3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TileRow {
    pub tiles: BitArr!(for 16, in u8, Msb0),
}

pub type Tile = [TileRow; 8];

pub const fn empty_tile() -> Tile {
    [TileRow {
        tiles: BitArray::ZERO,
    }; 8]
}

pub fn from_bytes_row(bytes: [u8; 2]) -> TileRow {
    let tiles = BitArray::new(bytes);
    TileRow { tiles }
}

pub fn from_bytes_tile(bytes: [u8; 16]) -> Tile {
    bytes
        .chunks_exact(2)
        .map(|bytes| from_bytes_row(bytes.try_into().unwrap()))
        .collect::<Vec<_>>()
        .try_into()
        .unwrap()
}

impl TileRow {
    pub fn get_tile(self, tile: u8) -> Pixel {
        let tile = usize::from(tile);
        let lsb = u8::from(self.tiles[tile]);
        let msb = u8::from(self.tiles[tile + 8]);
        Pixel::from_u8(msb << 1 | lsb).unwrap()
    }

    pub const fn iter(&self) -> TileRowIterator {
        TileRowIterator {
            tile_row: self,
            index: 0,
        }
    }
}

pub struct TileRowIterator<'a> {
    pub tile_row: &'a TileRow,
    pub index: u8,
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
