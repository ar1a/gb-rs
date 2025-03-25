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

    const fn af(&self) -> u16 {
        u16::from_le_bytes([self.a, self.f])
    }
    const fn set_af(&mut self, value: u16) {
        let [a, f] = value.to_le_bytes();
        self.a = a;
        self.f = f;
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
