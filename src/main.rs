mod cpu;

fn main() {
    let boot_rom = include_bytes!("../dmg_boot.bin");
    println!("Hello, world!");
}
