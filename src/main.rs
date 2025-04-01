use jane_eyre::eyre::{self, eyre};
use minifb::{Key, Window, WindowOptions};

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
    let mut buffer: Vec<u32> = vec![from_u8_rgb(0, 127, 255); WIDTH * HEIGHT];
    let mut window = Window::new("gb-rs", WIDTH, HEIGHT, WindowOptions::default())
        .map_err(|x| eyre!("{x:?}"))?;
    window.set_target_fps(60);

    while window.is_open() && !window.is_key_down(Key::Escape) {
        // buffer.fill(255_255_255);

        window.update_with_buffer(&buffer, WIDTH, HEIGHT)?;
    }

    println!("Hello, world!");
    Ok(())
}
