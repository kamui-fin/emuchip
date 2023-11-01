// 16 8-bit data registers named V0 to VF
// I -> address register (12 bits)
//
// Delay timer & Sound timer: Count down at 60 times / s until 0
// Beep when sound timer is non-zero
//
// Display res: 64 width, 32 height
//
// 35 opcodes, each are 2 bytes (big-endian)
//      NNN: address
//      NN: 8-bit constant
//      N: 4-bit constant
//      X and Y: 4-bit register identifier

// TODO: fix unsigned integer sizes inconsistency
//
// Separately:
// CPU: 700 times per second
// Display: 60 times per second
// Timer: 60 times per second

mod decode;
mod display;
mod emulator;
mod keyboard;
mod memory;
mod registers;
mod sound;

use std::{thread, time::Duration};

use emulator::Emulator;

fn main() {
    let mut emu = Emulator::init();
    while emu.is_running() {
        for _ in 0..10 {
            emu.tick();
        }
        emu.sync();
        thread::sleep(Duration::from_millis(16));
    }
}
