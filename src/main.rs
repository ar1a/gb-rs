#![feature(thread_sleep_until)]
use std::{
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use jane_eyre::eyre::{self, eyre};
use minifb::{Key, Window, WindowOptions};
use tracing::info;
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

use crate::{
    cpu::Cpu,
    gpu::{HEIGHT, Mode, WIDTH},
};

mod cpu;
mod disassembler;
mod gpu;

const fn from_u8_rgb(r: u8, g: u8, b: u8) -> u32 {
    u32::from_be_bytes([0, r, g, b])
}

fn main() -> eyre::Result<()> {
    jane_eyre::install()?;
    tracing_subscriber::registry()
        .with(tracing_subscriber::fmt::layer().with_writer(std::io::stderr))
        .with(
            EnvFilter::builder()
                .with_default_directive("info".parse()?)
                .from_env_lossy(),
        )
        .init();
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

        let cycles_per_second = 4_190_000;
        let target_cycles = cycles_per_second / 60;

        let mut next_frame = Instant::now();
        let mut last_mode = cpu.bus.gpu.mode;
        while cpu.pc < 0x100 {
            // do 60 bursts of cycles per second
            info!(delta = ?(next_frame - Instant::now()), target = ?(Duration::from_secs_f64(1.0/60.0)), "frame took");
            std::thread::sleep_until(next_frame);
            next_frame += Duration::from_secs_f64(1.0 / 60.0);
            let mut cycles_elapsed = 0;
            while cycles_elapsed < target_cycles {
                let cycles = cpu.step();
                cycles_elapsed += u32::from(cycles);

                if cpu.bus.gpu.mode == Mode::HBlank && last_mode != Mode::HBlank {
                    let mut buffer = buffer.lock().unwrap();
                    buffer
                        .iter_mut()
                        .skip(cpu.bus.gpu.line as usize * WIDTH)
                        .zip(
                            cpu.bus
                                .gpu
                                .buffer
                                .chunks_exact(3)
                                .skip(cpu.bus.gpu.line as usize * WIDTH),
                        )
                        // .take(WIDTH)
                        .for_each(|(x, rgb)| *x = from_u8_rgb(rgb[0], rgb[1], rgb[2]));
                }
                last_mode = cpu.bus.gpu.mode;
            }
        }
    });

    let _ = gui_thread.join();
    // let _ = emu_thread.join();
    // if `gui_thread` has ended it means we should just kill the emulator

    Ok(())
}
