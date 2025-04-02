use bitvec::array::BitArray;
use enumflags2::BitFlag;

use crate::gpu::{Gpu, LCDControl, VRAM_BEGIN, VRAM_END};

pub const BOOT_ROM_BEGIN: usize = 0x00;
pub const BOOT_ROM_END: usize = 0xFF;
pub const BOOT_ROM_SIZE: usize = BOOT_ROM_END - BOOT_ROM_BEGIN + 1;

pub const ROM_BANK_0_BEGIN: usize = 0x0000;
pub const ROM_BANK_0_END: usize = 0x3FFF;
pub const ROM_BANK_0_SIZE: usize = ROM_BANK_0_END - ROM_BANK_0_BEGIN + 1;

pub const ROM_BANK_N_BEGIN: usize = 0x4000;
pub const ROM_BANK_N_END: usize = 0x7FFF;
pub const ROM_BANK_N_SIZE: usize = ROM_BANK_N_END - ROM_BANK_N_BEGIN + 1;

pub const IO_BEGIN: usize = 0xFF00;
pub const IO_END: usize = 0xFF7F;
pub const IO_SIZE: usize = IO_END - IO_BEGIN + 1;

pub const HRAM_BEGIN: usize = 0xFF80;
pub const HRAM_END: usize = 0xFFFE;
pub const HRAM_SIZE: usize = HRAM_END - HRAM_BEGIN + 1;

#[derive(Debug)]
pub struct MemoryBus {
    boot_rom: Option<Box<[u8; BOOT_ROM_SIZE]>>,
    rom_bank_0: Box<[u8; ROM_BANK_0_SIZE]>,
    rom_bank_n: Box<[u8; ROM_BANK_N_SIZE]>,
    pub gpu: Gpu,
    hram: Box<[u8; HRAM_SIZE]>,
}

impl MemoryBus {
    pub fn new(boot_rom: Option<&[u8]>, game_rom: &[u8]) -> Self {
        let boot_rom = boot_rom.map(|rom| {
            rom.to_owned()
                .into_boxed_slice()
                .try_into()
                .unwrap_or_else(|_| {
                    panic!("Boot ROM to be size {BOOT_ROM_SIZE} (is {})", rom.len())
                })
        });
        Self {
            gpu: Gpu::default(),
            boot_rom,
            rom_bank_0: game_rom[..ROM_BANK_0_SIZE]
                .to_owned()
                .into_boxed_slice()
                .try_into()
                .expect("ROM to have bank 0"),
            rom_bank_n: game_rom[ROM_BANK_0_SIZE..ROM_BANK_0_SIZE + ROM_BANK_N_SIZE]
                .to_owned()
                .into_boxed_slice()
                .try_into()
                .expect("ROM to have bank n"),
            hram: vec![0; HRAM_SIZE].into_boxed_slice().try_into().unwrap(),
        }
    }
}

impl MemoryBus {
    pub fn read_byte(&self, address: u16) -> u8 {
        const ROM_BANK_0_BEGIN: usize = BOOT_ROM_END + 1; // shadowed so that the match statement
        // doesn't have overlapping ranges

        let address = address as usize;
        match address {
            BOOT_ROM_BEGIN..=BOOT_ROM_END => self
                .boot_rom
                .as_ref()
                .map_or_else(|| self.rom_bank_0[address], |boot_rom| boot_rom[address]),
            ROM_BANK_0_BEGIN..=ROM_BANK_0_END => self.rom_bank_0[address],
            ROM_BANK_N_BEGIN..=ROM_BANK_N_END => self.rom_bank_n[address - ROM_BANK_N_BEGIN],
            VRAM_BEGIN..=VRAM_END => self.gpu.read_vram(address - VRAM_BEGIN),
            IO_BEGIN..=IO_END => self.read_io_register(address),
            HRAM_BEGIN..HRAM_END => self.hram[address - HRAM_BEGIN],
            _ => todo!("memory region not readable yet: {:#4x}", address),
        }
    }
    pub fn write_byte(&mut self, address: u16, value: u8) {
        let address = address as usize;
        match address {
            ROM_BANK_0_BEGIN..=ROM_BANK_N_END => panic!("attempted to write to ROM"),
            VRAM_BEGIN..=VRAM_END => self.gpu.write_vram(address - VRAM_BEGIN, value),
            IO_BEGIN..=IO_END => self.write_io_register(address, value),
            HRAM_BEGIN..HRAM_END => self.hram[address - HRAM_BEGIN] = value,
            _ => todo!("memory region not writable yet: {:#4x}", address),
        }
    }

    pub fn read_word(&self, address: u16) -> u16 {
        let bytes = [self.read_byte(address), self.read_byte(address + 1)];
        u16::from_le_bytes(bytes)
    }
    pub fn write_word(&mut self, address: u16, value: u16) {
        let bytes = u16::to_le_bytes(value);
        self.write_byte(address, bytes[0]);
        self.write_byte(address + 1, bytes[1]);
    }

    #[allow(clippy::match_same_arms)]
    fn write_io_register(&mut self, address: usize, value: u8) {
        match address {
            0xFF11 => { /* Sound Ch1 Length Timer and Duty Cycle */ }
            0xFF12 => { /* Sound Ch1 Volume and Envelope */ }
            0xFF13 => { /* Sound Ch1 Period Low */ }
            0xFF14 => { /* Sound Ch1 Period High and Control */ }
            0xFF24 => { /* Master Volume and VIN panning */ }
            0xFF25 => { /* Sound Panning */ }
            0xFF26 => { /* Sound Enabled */ }
            0xFF40 => self.gpu.lcd_control = LCDControl::from_bits(value).unwrap(),
            0xFF42 => self.gpu.viewport_y_offset = value,
            0xFF47 => self.gpu.background_colours = BitArray::new([value]),
            _ => todo!("implement io register write {address:04X}"),
        }
    }

    fn read_io_register(&self, address: usize) -> u8 {
        match address {
            0xFF40 => self.gpu.lcd_control.bits(),
            0xFF42 => self.gpu.viewport_y_offset,
            0xFF44 => self.gpu.line,
            _ => todo!("implement io register read {address:04X}"),
        }
    }

    pub fn slice_from(&self, pc: u16) -> [u8; 4] {
        // TODO: iterator?
        [
            self.read_byte(pc),
            self.read_byte(pc + 1),
            self.read_byte(pc + 2),
            self.read_byte(pc + 3),
        ]
    }
}
