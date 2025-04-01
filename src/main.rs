use std::sync::{Arc, Mutex};

use jane_eyre::eyre::{self, eyre};
use minifb::{Key, Window, WindowOptions};

use crate::cpu::Cpu;

mod cpu;
mod disassembler;
mod gpu;

const WIDTH: usize = 160;
const HEIGHT: usize = 144;

const fn from_u8_rgb(r: u8, g: u8, b: u8) -> u32 {
    let (r, g, b) = (r as u32, g as u32, b as u32);
    (r << 16) | (g << 8) | b
}

fn main() -> eyre::Result<()> {
    jane_eyre::install()?;
    let buffer = Arc::new(Mutex::new(vec![from_u8_rgb(0, 127, 255); WIDTH * HEIGHT]));
    let gui_buffer = Arc::clone(&buffer);
    let gui_thread = std::thread::spawn(move || {
        let mut window = Window::new("gb-rs", WIDTH, HEIGHT, WindowOptions::default())
            .map_err(|x| eyre!("{x:?}"))
            .unwrap();
        window.set_target_fps(60);

        while window.is_open() && !window.is_key_down(Key::Escape) {
            // FIXME: copies 92KB 60 times a second...
            let buffer = gui_buffer.lock().unwrap().clone();
            window.update_with_buffer(&buffer, WIDTH, HEIGHT).unwrap();
        }
    });

    let _ = std::thread::spawn(move || {
        let boot_rom = include_bytes!("../dmg_boot.bin");
        let test_rom = include_bytes!("../test_roms/cpu_instrs/individual/01-special.gb");
        let mut cpu = Cpu::default();
        cpu.bus.slice_mut()[0..256].copy_from_slice(boot_rom);
        cpu.bus.slice_mut()[256..32768].copy_from_slice(&test_rom[256..]);
        while cpu.pc < 0x100 {
            cpu.step();
        }
    });

    let _ = gui_thread.join();
    // let _ = emu_thread.join();
    // if `gui_thread` has ended it means we should just kill the emulator

    Ok(())
}
