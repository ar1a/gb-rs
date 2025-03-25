/// Base registers
#[derive(Debug, Default)]
struct Registers {
    a: u8,
    b: u8,
    c: u8,
    d: u8,
    e: u8,
    f: u8,
    h: u8,
    l: u8,
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
}
