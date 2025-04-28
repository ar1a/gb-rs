use bilge::prelude::*;

#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone, Copy, Default)]
pub struct Joypad {
    pub input_select: UpperNibble,

    pub buttons: Buttons,
    pub dpad: Dpad,
}

#[bitsize(4)]
#[derive(DebugBits, Clone, Copy, FromBits, Default)]
pub struct UpperNibble {
    // bilge bitfields are LSB at the top
    select: NibbleSelect,
    reserved: u2,
}

#[bitsize(2)]
#[derive(Debug, Clone, Copy, FromBits, Default)]
// Variants are selected by setting the bit to 0
pub enum NibbleSelect {
    Button = 0b01,
    Dpad = 0b10,
    Both = 0b00,
    #[default]
    None = 0b11,
}

#[bitsize(4)]
#[derive(DebugBits, Clone, Copy, Default)]
pub struct Buttons {
    pub a: bool,
    pub b: bool,
    pub select: bool,
    pub start: bool,
}

#[bitsize(4)]
#[derive(DebugBits, Clone, Copy, Default)]
pub struct Dpad {
    pub right: bool,
    pub left: bool,
    pub up: bool,
    pub down: bool,
}

impl Joypad {
    pub fn write_joypad(&mut self, value: u8) {
        // lower nibble is read-only
        self.input_select = UpperNibble::from(u4::extract_u8(value, 4));
    }

    pub fn read_joypad(self) -> u8 {
        let upper = self.input_select.value.as_u8();
        let lower = match self.input_select.select() {
            NibbleSelect::Button => self.button_nibble(),
            NibbleSelect::Dpad => self.dpad_nibble(),
            // If both are selected, bits are set to 0 if *either* of the buttons assigned to that
            // bit are pressed
            NibbleSelect::Both => self.button_nibble() & self.dpad_nibble(),
            NibbleSelect::None => 0xF,
        };
        upper << 4 | lower
    }

    fn button_nibble(self) -> u8 {
        // invert the bits because a button being pressed is seen as that bit being 0
        // flip them before converting to a u8 so the upper nibble isn't touched
        (!self.buttons.value).as_u8()
    }
    fn dpad_nibble(self) -> u8 {
        // invert the bits because a button being pressed is seen as that bit being 0
        // flip them before converting to a u8 so the upper nibble isn't touched
        (!self.dpad.value).as_u8()
    }
}
