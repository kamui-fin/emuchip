// 16 8-bit data registers named V0 to VF
// I -> address register (12 bits)
//
// Implement stack storing return addresses
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

// TODO: to run older games from the 1970s or 1980s, consider making a configurable option in your emulator to toggle between these behaviors.

// TODO: fix unsigned integer sizes inconsistency

use std::time::Instant;

use decode::OpCodes;
use emulator::Emulator;

mod decode;
mod display;
mod emulator;
mod memory;
mod registers;

// Separately:
// CPU: 700 times per second
// Display: 60 times per second
// Timer: 60 times per second

fn main() {
    let mut emu = Emulator::init();
    while emu.is_running() {
        if emu.can_execute() {
            let operation = emu.fetch_decode();
            emu.execute_ins(operation);
        }
        emu.sync_timers();
        emu.sync_display();
    }
}
