#![allow(dead_code)]

use bitvec::{BitArr, array::BitArray, order::Msb0};
use enumflags2::{BitFlags, bitflags};
use num_derive::FromPrimitive;

use crate::gpu::tile::{Colour, Tile, TileRow, empty_tile};

pub const VRAM_BEGIN: usize = 0x8000;
pub const VRAM_END: usize = 0x9FFF;
pub const VRAM_SIZE: usize = VRAM_END - VRAM_BEGIN + 1;

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
    WindowTileMap = 1 << 6,
    WindowEnabled = 1 << 5,
    TileDataSelect = 1 << 4,
    BackgroundTileMap = 1 << 3,
    TallSprites = 1 << 2,
    SpritesEnabled = 1 << 1,
    BackgroundEnabled = 1 << 0,
}

#[derive(Debug)]
pub struct Gpu {
    vram: [u8; VRAM_SIZE],
    tile_set: [Tile; 384],
    pub buffer: Box<[u8; WIDTH * HEIGHT * 3]>,
    cycles: u16,
    pub line: u8,
    pub mode: Mode,

    pub lcd_control: BitFlags<LCDControl>,
    pub background_colours: BitArr!(for 8, in u8, Msb0),
    pub scroll_y: u8,
    pub scroll_x: u8,
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

    #[allow(clippy::similar_names)]
    fn render_line(&mut self) {
        const fn lookup_colour(pixel: Colour) -> (u8, u8, u8) {
            // TODO: implement
            match pixel {
                Colour::Three => (0, 0, 0),
                Colour::Two => (255 / 3, 255 / 3, 255 / 3),
                Colour::One => (255 / 2, 255 / 2, 255 / 2),
                Colour::Zero => (255, 255, 255),
            }
        }
        let mut tile_x_index = self.scroll_x / 8;
        let tile_y_index = self.line.wrapping_add(self.scroll_y);

        // width of entire background is 32 tiles
        let tile_offset = (u16::from(tile_y_index) / 8) * 32u16;

        let background_tile_map = if self.lcd_control.contains(LCDControl::BackgroundTileMap) {
            0x9C00
        } else {
            0x9800
        };
        let tile_map_begin = background_tile_map - VRAM_BEGIN;
        let tile_map_offset = tile_map_begin + tile_offset as usize;

        let row_y_offset = tile_y_index % 8;
        let mut pixel_x_index = self.scroll_x % 8;

        self.buffer
            .chunks_exact_mut(3)
            .skip(self.line as usize * WIDTH)
            .take(WIDTH)
            .for_each(|buf| {
                let tile_index = self.vram[tile_map_offset + tile_x_index as usize];
                let tile_value = self.tile_set[tile_index as usize][row_y_offset as usize]
                    .get_colour(pixel_x_index);
                let (r, g, b) = lookup_colour(tile_value);
                buf[0] = r;
                buf[1] = g;
                buf[2] = b;
                pixel_x_index = (pixel_x_index + 1) % 8;
                if pixel_x_index == 0 {
                    tile_x_index += 1;
                }
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
