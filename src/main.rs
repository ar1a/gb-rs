#![feature(thread_sleep_until)]
use std::{
    fs::File,
    io::{BufWriter, Write as _},
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};

use clap::Parser;
use jane_eyre::eyre::{self, eyre};
use minifb::{Key, Window, WindowOptions};
use tracing::{debug, warn};
use tracing_subscriber::{EnvFilter, layer::SubscriberExt, util::SubscriberInitExt};

use crate::{
    cpu::Cpu,
    gpu::{HEIGHT, Mode, WIDTH},
};

mod cpu;
mod disassembler;
mod gpu;
mod joypad;

mod timer;

const fn from_u8_rgb(r: u8, g: u8, b: u8) -> u32 {
    let (r, g, b) = (r as u32, g as u32, b as u32);
    (r << 16) | (g << 8) | b
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    log: bool,
    #[arg(short, long)]
    use_boot_rom: bool,
    #[arg(short, long)]
    fast: bool,
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

    let args = Args::parse();

    let buffer = Arc::new(Mutex::new(vec![0; WIDTH * HEIGHT * 3]));
    let gui_buffer = Arc::clone(&buffer);
    let gui_thread = std::thread::spawn(move || {
        let mut window = Window::new("gb-rs", WIDTH, HEIGHT, WindowOptions::default())
            .map_err(|x| eyre!("{x:?}"))
            .unwrap();
        window.set_target_fps(60);

        while window.is_open() && !window.is_key_down(Key::Escape) {
            // FIXME: copies 92KB 60 times a second...
            let buffer: Vec<u32> = gui_buffer
                .lock()
                .unwrap()
                .clone() // releases the lock
                .chunks_exact(3)
                .map(|rgb| from_u8_rgb(rgb[0], rgb[1], rgb[2]))
                .collect();
            window.update_with_buffer(&buffer, WIDTH, HEIGHT).unwrap();
        }
    });

    let _ = std::thread::spawn(move || {
        let boot_rom = if args.use_boot_rom {
            Some(include_bytes!("../dmg_boot.bin"))
        } else {
            None
        };
        let test_rom = include_bytes!("../test_roms/mem_timing/individual/01-read_timing.gb");
        let mut cpu = Cpu::new(boot_rom, test_rom, args.log);
        let mut f = if args.log {
            Some(BufWriter::new(File::create("log.txt").unwrap()))
        } else {
            None
        };

        if args.log {
            // log initial state
            f.as_mut()
                .unwrap()
                .write_all(&cpu.format_state().into_bytes())
                .unwrap_or_else(|e| warn!("failed to write to buffer {e}"));
        }

        let cycles_per_second = 4_190_000;
        let frame_duration = Duration::from_secs_f64(1.0 / 60.0);
        let target_cycles = cycles_per_second / 60;

        let mut next_frame = Instant::now() + frame_duration;
        let mut last_mode = cpu.bus.gpu.mode;
        loop {
            // do 60 bursts of cycles per second
            let mut cycles_elapsed = 0;
            while cycles_elapsed < target_cycles {
                let was_halted = cpu.halted;
                let cycles = cpu.step();
                cycles_elapsed += u32::from(cycles);

                if args.log && cycles > 0 && cpu.pc != 0x50 && !(was_halted && cpu.halted) {
                    f.as_mut()
                        .unwrap()
                        .write_all(&cpu.format_state().into_bytes())
                        .unwrap_or_else(|e| warn!("failed to write to buffer {e}"));
                }

                if cpu.bus.gpu.mode == Mode::HBlank && last_mode != Mode::HBlank {
                    let mut buffer = buffer.lock().unwrap();
                    buffer.copy_from_slice(&*cpu.bus.gpu.buffer);
                }
                last_mode = cpu.bus.gpu.mode;
            }

            if args.log {
                // flush after every 1/60th burst
                f.as_mut()
                    .unwrap()
                    .flush()
                    .unwrap_or_else(|e| warn!("failed to flush to file {e}"));
            }

            debug!(
                delta = ?(frame_duration.saturating_sub(next_frame.duration_since(Instant::now()))),
                target = ?frame_duration,
                "frame took"
            );
            if !next_frame.elapsed().is_zero() {
                warn!("lagging by {:?}", next_frame.elapsed());
            }

            if !args.fast {
                std::thread::sleep_until(next_frame);
            }
            next_frame = Instant::now() + frame_duration;
        }
    });

    let _ = gui_thread.join();
    // let _ = emu_thread.join();
    // if `gui_thread` has ended it means we should just kill the emulator

    Ok(())
}
