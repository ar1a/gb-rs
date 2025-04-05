#![allow(dead_code)]

use bitvec::{BitArr, array::BitArray, order::Lsb0};
use enumflags2::{BitFlags, bitflags};
use num_derive::FromPrimitive;

use crate::gpu::tile::{ColourIndex, Tile, TileRow, empty_tile};

pub const VRAM_BEGIN: usize = 0x8000;
pub const VRAM_END: usize = 0x9FFF;
pub const VRAM_SIZE: usize = VRAM_END - VRAM_BEGIN + 1;

pub const OAM_BEGIN: usize = 0xFE00;
pub const OAM_END: usize = 0xFE9F;
pub const OAM_SIZE: usize = OAM_END - OAM_BEGIN + 1;

pub const WIDTH: usize = 160;
pub const HEIGHT: usize = 144;

pub mod tile;

#[derive(Debug, Clone, Copy)]
enum Palette {
    Zero = 0,
    One = 1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    OamScan = 2,
    Drawing = 3,
    HBlank = 0,
    VBlank = 1,
}

#[derive(Debug, Clone, Copy, FromPrimitive)]
enum TileMapSelect {
    X9800 = 0,
    X9C00 = 1,
}

#[bitflags]
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LCDControl {
    DisplayEnabled = 1 << 7,
    /// On = 0x9C00, Off = 0x9800
    WindowTileMap = 1 << 6,
    WindowEnabled = 1 << 5,
    /// On = 0x8000, Off = 0x8800
    TileDataSelect = 1 << 4,
    // On = 0x9C00, Off = 0x9800
    BackgroundTileMap = 1 << 3,
    TallSprites = 1 << 2,
    SpritesEnabled = 1 << 1,
    BackgroundEnabled = 1 << 0,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Gpu {
    vram: [u8; VRAM_SIZE],
    oam: [u8; OAM_SIZE],
    tile_set: [Tile; 384],
    pub buffer: Box<[u8; WIDTH * HEIGHT * 3]>,
    cycles: u16,
    pub line: u8,
    pub mode: Mode,

    pub lcd_control: BitFlags<LCDControl>,
    pub background_colours: BitArr!(for 8, in u8, Lsb0),
    pub scroll_y: u8,
    pub scroll_x: u8,
}

trait LCDExt {
    fn bg_tilemap_address(&self) -> usize;
    fn tile_data_address(&self) -> usize;
    fn window_tilemap_address(&self) -> usize;
}
impl LCDExt for BitFlags<LCDControl> {
    fn bg_tilemap_address(&self) -> usize {
        if self.contains(LCDControl::BackgroundTileMap) {
            0x9C00
        } else {
            0x9800
        }
    }

    fn window_tilemap_address(&self) -> usize {
        if self.contains(LCDControl::WindowTileMap) {
            0x9C00
        } else {
            0x9800
        }
    }

    fn tile_data_address(&self) -> usize {
        if self.contains(LCDControl::TileDataSelect) {
            0x8000
        } else {
            0x8800
        }
    }
}

impl Default for Gpu {
    fn default() -> Self {
        Self {
            vram: [0; VRAM_SIZE],
            oam: [0; OAM_SIZE],
            tile_set: [empty_tile(); 384],
            buffer: vec![0; WIDTH * HEIGHT * 3]
                .into_boxed_slice()
                .try_into()
                .unwrap(),
            cycles: 0,
            line: 0,
            mode: Mode::HBlank,
            lcd_control: BitFlags::EMPTY,
            background_colours: BitArray::ZERO,
            scroll_y: 0,
            scroll_x: 0,
        }
    }
}

impl Gpu {
    pub fn step(&mut self, cycles: u8) {
        if !self.lcd_control.contains(LCDControl::DisplayEnabled) {
            return;
        }
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
                    self.line += 1;

                    if self.line >= 154 {
                        self.mode = Mode::OamScan;
                        self.line = 0;
                    }
                }
            }
        }
    }
    pub const fn read_vram(&self, index: usize) -> u8 {
        self.vram[index]
    }

    pub fn write_vram(&mut self, index: usize, value: u8) {
        self.vram[index] = value;
        // if we're not writing to the tile set storage, return early
        if index >= 0x1800 {
            return;
        }

        // tile rows are encoded as 2 bytes, with the first byte always on an even address. This
        // ignores the last bit, so the index is always an even number.
        let normalized_index = index & (!1);

        let tile_row = TileRow::from_bytes(
            self.vram[normalized_index..=normalized_index + 1]
                .try_into()
                .unwrap(),
        );

        let tile_index = index / 16;
        let row_index = (index % 16) / 2;

        self.tile_set[tile_index][row_index] = tile_row;
    }

    pub const fn read_oam(&self, address: usize) -> u8 {
        self.oam[address]
    }

    pub const fn write_oam(&mut self, address: usize, value: u8) {
        self.oam[address] = value;
    }

    #[allow(clippy::cast_possible_truncation, clippy::similar_names)]
    fn render_line(&mut self) {
        let lookup_colour = |pixel: ColourIndex| -> (u8, u8, u8) {
            let bit = pixel as usize * 2;
            let value = &self.background_colours[bit..=bit + 1];
            let value = u8::from(value[0]) << 1 | u8::from(value[1]);
            match value {
                0 => (255, 255, 255),
                1 => (170, 170, 170),
                2 => (85, 85, 85),
                3 => (0, 0, 0),
                _ => unreachable!(),
            }
        };
        let tile_x_coordinate = usize::from(self.scroll_x / 8); // FIXME: Wrapping might be broken
        let tile_y_coordinate = self.line.wrapping_add(self.scroll_y);
        let background_tile_map = self.lcd_control.bg_tilemap_address();
        let offset = 32 * (usize::from(tile_y_coordinate) / 8);
        let address = background_tile_map - VRAM_BEGIN + offset;

        let pixels = self.vram[address + tile_x_coordinate..]
            .iter()
            .map(|tile_number| {
                &self.tile_set[usize::from(*tile_number)][usize::from(tile_y_coordinate) % 8]
            })
            .flat_map(|row| row.iter())
            .skip(usize::from(self.scroll_x) % 8);

        self.buffer
            .chunks_exact_mut(3)
            .skip(self.line as usize * WIDTH)
            .take(WIDTH)
            .zip(pixels)
            .for_each(|(buf, pixel)| {
                let (r, g, b) = lookup_colour(pixel);
                buf[0] = r;
                buf[1] = g;
                buf[2] = b;
            });
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
