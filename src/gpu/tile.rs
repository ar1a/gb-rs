use bitvec::{BitArr, array::BitArray, order::Msb0};

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

pub fn from_bytes_tile(bytes: [u8; 16]) -> Tile {
    bytes
        .chunks_exact(2)
        .map(|bytes| TileRow::from_bytes(bytes.try_into().unwrap()))
        .collect::<Vec<_>>()
        .try_into()
        .unwrap()
}

pub type ColourIndex = u8;
impl TileRow {
    pub fn get_colour(self, tile: u8) -> ColourIndex {
        let tile = usize::from(tile);
        let lsb = u8::from(self.tiles[tile]);
        let msb = u8::from(self.tiles[tile + 8]);
        msb << 1 | lsb
    }

    pub const fn iter(&self) -> TileRowIterator {
        TileRowIterator {
            tile_row: self,
            index: 0,
        }
    }

    pub fn from_bytes(bytes: [u8; 2]) -> Self {
        let tiles = BitArray::new(bytes);
        Self { tiles }
    }
}

pub struct TileRowIterator<'a> {
    pub tile_row: &'a TileRow,
    pub index: u8,
}

impl Iterator for TileRowIterator<'_> {
    type Item = ColourIndex;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < 8 {
            let result = Some(self.tile_row.get_colour(self.index));
            self.index += 1;
            result
        } else {
            None
        }
    }
}
