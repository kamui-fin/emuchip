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

use std::{fs, process, thread, time::Duration};

mod display;

type Addr = u16; // in reality u12

/* Low level emulation mappers */

struct Memory {
    // 4k bytes
    // font data stored from 050 -> 09F (000 -> 04F is empty by convention)
    bytes: [u8; 4096],
    pc: ProgramCounter,
}

impl Memory {
    fn new() -> Self {
        Self {
            bytes: [0; 4096],
            pc: ProgramCounter(0x09F, 0),
        }
    }

    fn next_instruction(&mut self) -> u16 {
        let (l, r) = (
            self.bytes[self.pc.0 as usize],
            self.bytes[(self.pc.0 + 1) as usize],
        );
        let result = self.pc.increment();
        if !result {
            process::exit(0);
        }
        ((l as u16) << 8) | r as u16
    }

    // loads program instructions starting at address 0x09F
    fn load_rom(&mut self, bytes: &[u8]) {
        self.pc.set_end(bytes.len());
        let start_index = 0x09F;
        if start_index + bytes.len() <= 4096 {
            self.bytes[start_index..start_index + bytes.len()].copy_from_slice(bytes);
        }
    }

    fn load_rom_by_file(&mut self, path: &str) {
        let program = fs::read(path).unwrap();
        self.load_rom(program.as_slice());
    }
}

struct Stack {
    addresses: Vec<Addr>,
}

impl Stack {
    fn new() -> Self {
        Self { addresses: vec![] }
    }

    fn push(&mut self, addr: Addr) {
        self.addresses.push(addr)
    }

    fn pop(&mut self) -> Option<Addr> {
        self.addresses.pop()
    }
}

/* High level abstractions */

type FontBytes = [u8; 5 * 16];

const DEFAULT_FONT: FontBytes = [
    0xF0, 0x90, 0x90, 0x90, 0xF0, // 0
    0x20, 0x60, 0x20, 0x20, 0x70, // 1
    0xF0, 0x10, 0xF0, 0x80, 0xF0, // 2
    0xF0, 0x10, 0xF0, 0x10, 0xF0, // 3
    0x90, 0x90, 0xF0, 0x10, 0x10, // 4
    0xF0, 0x80, 0xF0, 0x10, 0xF0, // 5
    0xF0, 0x80, 0xF0, 0x90, 0xF0, // 6
    0xF0, 0x10, 0x20, 0x40, 0x40, // 7
    0xF0, 0x90, 0xF0, 0x90, 0xF0, // 8
    0xF0, 0x90, 0xF0, 0x10, 0xF0, // 9
    0xF0, 0x90, 0xF0, 0x90, 0x90, // A
    0xE0, 0x90, 0xE0, 0x90, 0xE0, // B
    0xF0, 0x80, 0x80, 0x80, 0xF0, // C
    0xE0, 0x90, 0x90, 0x90, 0xE0, // D
    0xF0, 0x80, 0xF0, 0x80, 0xF0, // E
    0xF0, 0x80, 0xF0, 0x80, 0x80, // F
];

struct Font {
    data: FontBytes,
}

impl Default for Font {
    fn default() -> Self {
        Self { data: DEFAULT_FONT }
    }
}

struct Timer {
    count: u8,
}

impl Timer {
    fn new(init_count: u8) -> Self {
        Self { count: init_count }
    }

    // Returns true if tick() sets to 0, else false
    fn tick(&mut self) -> bool {
        self.count -= 1;
        self.count == 0
    }
}

enum Register {
    // General purpose variable registers from 0 - F
    V0(),
    V1(),
    V2(),
    V3(),
    V4(),
    V5(),
    V6(),
    V7(),
    V8(),
    V9(),
    VA(),
    VB(),
    VC(),
    VD(),
    VE(),
    VF(),
}

#[derive(Debug)]
struct VariableRegister {
    label: u8, // in reality u4 (0 - F)
    data: u8,
}

// Special registers
#[derive(Debug)]
struct ProgramCounter(Addr, Addr);

impl ProgramCounter {
    fn increment(&mut self) -> bool {
        self.0 += 2;
        self.0 <= self.1
    }

    fn set_end(&mut self, len: usize) {
        self.1 = self.0 + (len as u16);
    }
}

struct IndexRegister(Addr);

struct RawInstruction(u16);

impl RawInstruction {
    fn nth_m_digits(&self, n: u8, m: u8) -> u16 {
        // 0110 1100 1111 0001
        // -------------------
        // 1111 1111 1111 1111
        //      1111 1111 1111
        //           1111 1111
        //                1111
        //
        // 4 - (1) - (n - 1), n = 2
        // 4 - (m) - (n - 1), n = 2
        let shift_places = (4 - m - (n - 1)) * 4;
        let mut mask = 0;
        for _ in 0..m {
            mask = (mask << 4) | 0b1111;
        }
        (self.0 & (mask << shift_places)) >> shift_places
    }
}

impl PartialEq<u16> for RawInstruction {
    fn eq(&self, ins: &u16) -> bool {
        ins.eq(&self.0)
    }
}

#[test]
fn test_bit_manip() {
    assert_eq!(RawInstruction(0x4CEE).nth_m_digits(2, 1), 0xC);
    assert_eq!(RawInstruction(0x4CEE).nth_m_digits(3, 1), 0xE);
    assert_eq!(RawInstruction(0x4CEE).nth_m_digits(1, 1), 0x4);

    assert_eq!(RawInstruction(0x4CEE).nth_m_digits(1, 2), 0x4C);
    assert_eq!(RawInstruction(0x4CEE).nth_m_digits(2, 2), 0xCE);
}

enum OpCodes {
    // 00E0
    // turn all pixels to 0
    ClearScreen,
    // 1NNN
    // set PC to address NNN, "jump" to memory location
    Jump(Addr),
    // 6XNN
    // set register VX to value NN
    SetRegister(Addr, u16),
    // 7XNN
    // add value NN to VX
    AddToRegister(Addr, u16),
    // ANNN
    // set index register I to address NNNN
    SetIndexRegister(Addr),
    // DXYN (hardest)
    // draw an N pixel tall sprite starting at I
    // at Coordinates (VX, VY)
    // XOR pixels on screen using sprite data
    // if pixels on screen were switched OFF: VF set to 1
    Display(Addr, Addr, u16),

    Unimplemented,
}

impl OpCodes {
    fn decode_raw(ins: u16) -> Self {
        let raw = RawInstruction(ins);
        if raw == 0x00E0 {
            return Self::ClearScreen;
        } else if raw.nth_m_digits(1, 1) == 0x1 {
            return Self::Jump(raw.nth_m_digits(2, 3));
        } else if raw.nth_m_digits(1, 1) == 0x6 {
            return Self::SetRegister(raw.nth_m_digits(2, 1), raw.nth_m_digits(3, 2));
        } else if raw.nth_m_digits(1, 1) == 0x7 {
            return Self::AddToRegister(raw.nth_m_digits(2, 1), raw.nth_m_digits(3, 2));
        } else if raw.nth_m_digits(1, 1) == 0xA {
            return Self::SetIndexRegister(raw.nth_m_digits(2, 3));
        } else if raw.nth_m_digits(1, 1) == 0xD {
            return Self::Display(
                raw.nth_m_digits(2, 1),
                raw.nth_m_digits(3, 1),
                raw.nth_m_digits(4, 1),
            );
        } else {
            return Self::Unimplemented;
        }
    }
}

fn main() {
    // display::start_display();

    const INS_PER_SECOND: u64 = 700;

    let mut mem = Memory::new();
    // let index_register = IndexRegister(0x0);

    mem.load_rom(include_bytes!("../roms/IBM Logo.ch8"));

    // main loop (700 CHIP-8 instructions per second)
    loop {
        let ins = mem.next_instruction();
        println!("{:#06x}", ins);

        // fetch:
        //  read ins @ PC (2 bytes)
        //  increment PC by 2 bytes
        // decode
        //  extract variables
        // execute
        //  run instruction

        // simulate OG hardware
        thread::sleep(Duration::from_millis(1_000 / INS_PER_SECOND));
    }
}
