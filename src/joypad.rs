#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone, Copy, Default)]
pub struct Joypad {
    pub register: u8,

    pub start: bool,
    pub select: bool,
    pub b: bool,
    pub a: bool,

    pub down: bool,
    pub up: bool,
    pub left: bool,
    pub right: bool,
}

impl Joypad {
    pub const fn write_joypad(&mut self, value: u8) {
        // lower nibble is read-only
        let mask = 0b1111_0000;

        self.register |= value & mask;
    }

    pub fn read_joypad(&self) -> u8 {
        let upper = self.register & 0b1111_0000;
        let lower = match (self.register >> 4) & 0b11 {
            0b01 => self.button_nibble(),
            0b10 => self.dpad_nibble(),
            0b11 => 0xF,
            0b00 => todo!("buttons and dpad selected for joypad read"),
            _ => unreachable!(),
        };
        upper | lower
    }

    const fn button_nibble(&self) -> u8 {
        let start = (self.start as u8) << 3;
        let select = (self.select as u8) << 2;
        let b = (self.b as u8) << 1;
        let a = self.a as u8;

        start | select | b | a
    }
    const fn dpad_nibble(&self) -> u8 {
        let down = (self.down as u8) << 3;
        let up = (self.up as u8) << 2;
        let left = (self.left as u8) << 1;
        let right = self.right as u8;

        down | up | left | right
    }
}
