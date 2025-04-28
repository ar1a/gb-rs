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
pub enum NibbleSelect {
    Button = 0b01,
    Dpad = 0b10,
    #[fallback]
    #[default]
    Reserved,
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

    // FIXME: Implement proper reading for if both buttons/dpad is selected
    // "The good news is you can actually select both buttons and directions by setting both
    // selection bits low. The resulting bits will be low if either the corresponding direction or
    // button is pressed."
    // <https://www.reddit.com/r/EmuDev/comments/zq6ygz/comment/j0yo0uh/>
    pub fn read_joypad(self) -> u8 {
        let upper: u8 = u4::from(self.input_select).into();
        let lower = match self.input_select.select() {
            NibbleSelect::Button => self.button_nibble(),
            NibbleSelect::Dpad => self.dpad_nibble(),
            NibbleSelect::Reserved => {
                todo!(
                    "handle incorrect joypad selection bits: {:04b}",
                    self.input_select.value
                );
            }
        };
        upper << 4 | lower
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
