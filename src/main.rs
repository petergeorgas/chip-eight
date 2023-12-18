mod drivers;
mod font;
mod processor;

use drivers::{DisplayDriver, InputDriver, Rom};
use processor::Processor;

const CHIP8_DISPLAY_WIDTH: usize = 64; // 64px wide
const CHIP8_DISPLAY_HEIGHT: usize = 32; // 32px tall
const CHIP8_MEMORY: usize = 4096; // 4 KB RAM asvailable

fn main() {
    let rom = Rom::new("roms/test_opcode.ch8");
    let sdl_context = sdl2::init().unwrap();
    let disp = DisplayDriver::new(&sdl_context);
    let inp = InputDriver::new(&sdl_context);

    let mut processor = Processor::new(disp, inp);

    processor.load_program(&rom.data);

    processor.start();
}
