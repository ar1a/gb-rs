use bilge::prelude::*;

#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone, Copy, Default)]
pub struct Joypad {
    pub register: u8,

    pub buttons: Buttons,
    pub dpad: Dpad,
}

#[bitsize(4)]
#[derive(DebugBits, Clone, Copy, Default)]
pub struct Buttons {
    pub start: bool,
    pub select: bool,
    pub b: bool,
    pub a: bool,
}

#[bitsize(4)]
#[derive(DebugBits, Clone, Copy, Default)]
pub struct Dpad {
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

    // TODO: Switch to <https://docs.rs/bilge/latest/bilge/>?
    pub fn read_joypad(self) -> u8 {
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

    fn button_nibble(self) -> u8 {
        // invert the bits because a button being pressed is seen as that bit being 0
        u8::from(!self.buttons.value)
    }
    fn dpad_nibble(self) -> u8 {
        // invert the bits because a button being pressed is seen as that bit being 0
        u8::from(!self.dpad.value)
    }
}
