use enumflags2::{BitFlag as _, BitFlags, bitflags};

/// Base registers
#[derive(Debug, Default)]
struct Registers {
    a: u8,
    b: u8,
    c: u8,
    d: u8,
    e: u8,
    h: u8,
    l: u8,

    f: BitFlags<Flags>,
}

#[bitflags]
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq)]
enum Flags {
    Zero = 0b1000_0000,
    Subtraction = 0b0100_0000,
    HalfCarry = 0b0010_0000,
    Carry = 0b0001_0000,
}

impl Registers {
    const fn bc(&self) -> u16 {
        u16::from_le_bytes([self.b, self.c])
    }
    const fn set_bc(&mut self, value: u16) {
        let [b, c] = value.to_le_bytes();
        self.b = b;
        self.c = c;
    }

    fn af(&self) -> u16 {
        u16::from_le_bytes([self.a, self.f.bits()])
    }
    fn set_af(&mut self, value: u16) {
        let [a, f] = value.to_le_bytes();
        self.a = a;
        self.f = Flags::from_bits(f).unwrap();
    }

    const fn de(&self) -> u16 {
        u16::from_le_bytes([self.d, self.e])
    }
    const fn set_de(&mut self, value: u16) {
        let [d, e] = value.to_le_bytes();
        self.d = d;
        self.e = e;
    }

    const fn hl(&self) -> u16 {
        u16::from_le_bytes([self.h, self.l])
    }
    const fn set_hl(&mut self, value: u16) {
        let [h, l] = value.to_le_bytes();
        self.h = h;
        self.l = l;
    }
}
