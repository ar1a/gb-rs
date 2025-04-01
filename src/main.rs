use jane_eyre::eyre::{self, eyre};
use minifb::{Key, Window, WindowOptions};

mod cpu;
mod disassembler;
mod gpu;

const WIDTH: usize = 160;
const HEIGHT: usize = 144;

fn main() -> eyre::Result<()> {
    jane_eyre::install()?;
    let mut buffer: Vec<u32> = vec![0; WIDTH * HEIGHT];
    let mut window = Window::new("gb-rs", WIDTH, HEIGHT, WindowOptions::default())
        .map_err(|x| eyre!("{x:?}"))?;
    window.set_target_fps(60);

    while window.is_open() && !window.is_key_down(Key::Escape) {
        buffer.fill(255_255_255);

        window.update_with_buffer(&buffer, WIDTH, HEIGHT)?;
    }

    println!("Hello, world!");
    Ok(())
}
