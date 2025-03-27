use enumflags2::{BitFlag as _, BitFlags, bitflags};
use parse_display::Display;

/// Base registers
#[derive(Debug, Default)]
pub(super) struct Registers {
    pub(super) a: u8,
    pub(super) b: u8,
    pub(super) c: u8,
    pub(super) d: u8,
    pub(super) e: u8,
    pub(super) h: u8,
    pub(super) l: u8,

    pub(super) f: BitFlags<Flags>,
}

#[bitflags]
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Display)]
pub(super) enum Flags {
    /// Set if the result of an operation is zero
    #[display("Z")]
    Zero = 0b1000_0000,
    /// Set if previous instruction was a Subtraction
    #[display("N")]
    Subtraction = 0b0100_0000,
    /// Set if lower 4 bits carried over to upper 4 bits
    #[display("H")]
    HalfCarry = 0b0010_0000,
    /// Set when something overflows
    #[display("C")]
    Carry = 0b0001_0000,
}

impl Registers {
    pub(super) const fn bc(&self) -> u16 {
        u16::from_le_bytes([self.c, self.b])
    }
    pub(super) const fn set_bc(&mut self, value: u16) {
        let [c, b] = value.to_le_bytes();
        self.b = b;
        self.c = c;
    }

    pub(super) fn af(&self) -> u16 {
        u16::from_le_bytes([self.f.bits(), self.a])
    }
    pub(super) fn set_af(&mut self, value: u16) {
        let [f, a] = value.to_le_bytes();
        self.a = a;
        self.f = Flags::from_bits(f).unwrap();
    }

    pub(super) const fn de(&self) -> u16 {
        u16::from_le_bytes([self.e, self.d])
    }
    pub(super) const fn set_de(&mut self, value: u16) {
        let [e, d] = value.to_le_bytes();
        self.d = d;
        self.e = e;
    }

    pub(super) const fn hl(&self) -> u16 {
        u16::from_le_bytes([self.l, self.h])
    }
    pub(super) const fn set_hl(&mut self, value: u16) {
        let [l, h] = value.to_le_bytes();
        self.h = h;
        self.l = l;
    }

    pub fn set_flag(&mut self, flag: Flags, condition: bool) -> String {
        self.f.set(flag, condition);
        format!("{}' = {}", flag, condition as u8)
    }
}
