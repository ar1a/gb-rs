#[derive(Debug)]
pub(crate) struct MemoryBus {
    memory: [u8; 0xFFFF],
}

impl Default for MemoryBus {
    fn default() -> Self {
        Self {
            memory: [0; 0xFFFF],
        }
    }
}

impl MemoryBus {
    pub fn read_byte(&self, address: u16) -> u8 {
        self.memory[address as usize]
    }
    pub fn write_byte(&mut self, address: u16, value: u8) {
        self.memory[address as usize] = value
    }

    pub fn read_word(&self, address: u16) -> u16 {
        let address = address as usize;
        u16::from_le_bytes(self.memory[address..=address + 1].try_into().unwrap())
    }
    pub fn write_word(&mut self, address: u16, value: u16) {
        let address = address as usize;
        let bytes = u16::to_le_bytes(value);
        self.memory[address..=address + 1].copy_from_slice(&bytes);
    }

    pub fn slice_from(&self, pc: u16) -> &[u8] {
        &self.memory[pc as usize..]
    }

    pub fn slice(&self) -> &[u8] {
        &self.memory
    }
    pub fn slice_mut(&mut self) -> &mut [u8] {
        &mut self.memory
    }
}
