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

use minifb::Key;
use rand::{self, Rng};

use crate::display::FrameBuffer;

mod display;

type TypeAddr = u16; // in reality u12
type TypeRegister = u8; // size of value contained in register AND register label itself

/* Low level emulation mappers */

struct Memory {
    // 4k bytes
    // font data stored from 050 -> 09F (000 -> 04F is empty by convention)
    bytes: [u8; 4096],
    pc: ProgramCounter,
    index: IndexRegister,
    font: Font,
    stack: Stack,
}

impl Memory {
    fn new() -> Self {
        Self {
            bytes: [0; 4096],
            pc: ProgramCounter(0x200, 0),
            index: IndexRegister(0x0),
            stack: Stack { addresses: vec![] },
            font: Font::default(),
        }
    }

    fn get(&self, addr: TypeAddr) -> u8 {
        self.bytes[addr as usize]
    }

    fn increment_pc(&mut self) {
        let result = self.pc.increment();
        if !result {
            process::exit(0);
        }
    }

    fn next_instruction(&mut self) -> u16 {
        let (l, r) = (
            self.bytes[self.pc.0 as usize],
            self.bytes[(self.pc.0 + 1) as usize],
        );
        self.increment_pc();
        ((l as u16) << 8) | r as u16
    }

    fn set_pc(&mut self, addr: TypeAddr) {
        self.pc.set_addr(addr);
    }

    fn set_index(&mut self, addr: TypeAddr) {
        self.index.set_addr(addr);
    }

    // loads program instructions starting at address 0x09F
    fn load_rom(&mut self, bytes: &[u8]) {
        self.pc.set_end(bytes.len());
        let start_index = 0x200;
        if start_index + bytes.len() <= 4096 {
            self.bytes[start_index..start_index + bytes.len()].copy_from_slice(bytes);
        }

        for i in start_index..start_index + bytes.len() {
            println!("{:03x?} = {:02x?}", i, self.bytes[i]);
        }
    }

    fn load_rom_by_file(&mut self, path: &str) {
        let program = fs::read(path).unwrap();
        self.load_rom(program.as_slice());
    }
}

struct Stack {
    addresses: Vec<TypeAddr>,
}

impl Stack {
    fn new() -> Self {
        Self { addresses: vec![] }
    }

    fn push(&mut self, addr: TypeAddr) {
        self.addresses.push(addr)
    }

    fn pop(&mut self) -> Option<TypeAddr> {
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

struct Registers {
    registers: [u8; 16],
}

impl Registers {
    fn new() -> Self {
        Self { registers: [0; 16] }
    }

    fn set_register(&mut self, reg_num: u8, value: u8) {
        self.registers[reg_num as usize] = value;
    }

    fn add_to_register(&mut self, reg_num: u8, value: u8) {
        self.registers[reg_num as usize] += value;
    }

    fn get(&self, reg_num: u8) -> u8 {
        self.registers[reg_num as usize]
    }
}

// Special registers
#[derive(Debug)]
struct ProgramCounter(TypeAddr, TypeAddr);

impl ProgramCounter {
    fn increment(&mut self) -> bool {
        self.0 += 2;
        self.0 <= self.1
    }

    fn set_end(&mut self, len: usize) {
        self.1 = self.0 + (len as u16);
    }

    fn set_addr(&mut self, addr: TypeAddr) {
        self.0 = addr;
    }
}

struct IndexRegister(TypeAddr);

impl IndexRegister {
    fn set_addr(&mut self, addr: TypeAddr) {
        self.0 = addr;
    }
}

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

#[derive(Debug)]
enum OpCodes {
    // 00E0
    // turn all pixels to 0
    ClearScreen,
    // 1NNN
    // set PC to address NNN, "jump" to memory location
    Jump(TypeAddr),
    // 6XNN
    // set register VX to value NN
    SetRegister(u8, u8),
    // 7XNN
    // add value NN to VX
    AddToRegister(u8, u8),
    // ANNN
    // set index register I to address NNNN
    SetIndexRegister(TypeAddr),
    // DXYN (hardest)
    // draw an N pixel tall sprite starting at I
    // at Coordinates (VX, VY)
    // XOR pixels on screen using sprite data
    // if pixels on screen were switched OFF: VF set to 1
    Display(u8, u8, u16),

    PushSubroutine(TypeAddr), // ok
    PopSubroutine,            // ok

    SkipEqualConstant(u8, u8),    // ok
    SkipNotEqualConstant(u8, u8), // ok
    SkipEqualRegister(u8, u8),    // ok
    SkipNotEqualRegister(u8, u8), // ok

    CopyRegister(u8, u8),     // ok
    Or(u8, u8),               // ok
    And(u8, u8),              // ok
    XOr(u8, u8),              // ok
    Add(u8, u8),              // ok
    SubtractForward(u8, u8),  // ok
    SubtractBackward(u8, u8), // ok
    LeftShift(u8, u8),        // ok
    RightShift(u8, u8),       // ok

    JumpWithOffset(TypeAddr), // ok
    Random(u8, u8),           // ok

    SkipIfPressed(u8),
    SkipIfNotPressed(u8),

    CopyDelayToRegister(u8),
    CopyRegisterToDelay(u8),
    CopyRegisterToSound(u8),

    AddToIndex(u8), // ok
    GetKey(u8),
    PointChar(u8),
    ToDecimal(u8),

    LoadRegisterFromMemory(u8),
    StoreRegisterToMemory(u8),

    Unimplemented,
}

impl OpCodes {
    fn decode_raw(ins: u16) -> Self {
        let raw = RawInstruction(ins);
        if raw == 0x00E0 {
            Self::ClearScreen
        } else if raw.nth_m_digits(1, 1) == 0x1 {
            Self::Jump(raw.nth_m_digits(2, 3))
        } else if raw.nth_m_digits(1, 1) == 0x6 {
            Self::SetRegister(raw.nth_m_digits(2, 1) as u8, raw.nth_m_digits(3, 2) as u8)
        } else if raw.nth_m_digits(1, 1) == 0x7 {
            Self::AddToRegister(raw.nth_m_digits(2, 1) as u8, raw.nth_m_digits(3, 2) as u8)
        } else if raw.nth_m_digits(1, 1) == 0xA {
            Self::SetIndexRegister(raw.nth_m_digits(2, 3))
        } else if raw.nth_m_digits(1, 1) == 0xD {
            Self::Display(
                raw.nth_m_digits(2, 1) as u8,
                raw.nth_m_digits(3, 1) as u8,
                raw.nth_m_digits(4, 1),
            )
        } else {
            Self::Unimplemented
        }
    }
}

fn main() {
    const INS_PER_SECOND: u64 = 700;

    let mut regs = Registers::new();
    let mut mem = Memory::new();

    mem.load_rom(include_bytes!("../roms/IBM Logo.ch8"));

    let mut fb = FrameBuffer::new();

    // main loop (700 CHIP-8 instructions per second)
    while fb.window.is_open() && !fb.window.is_key_down(Key::Escape) {
        // fetch:
        //  read ins @ PC (2 bytes)
        //  increment PC by 2 bytes
        // decode
        //  extract variables
        // execute
        //  run instruction
        let ins = mem.next_instruction();
        println!("Fetching {:#06x}", ins);
        let ins = OpCodes::decode_raw(ins);
        println!("Decoded to {:#?}", ins);

        match ins {
            OpCodes::Jump(addr) => {
                mem.set_pc(addr);
            }
            OpCodes::SetRegister(reg, val) => {
                regs.set_register(reg, val);
            }
            OpCodes::AddToRegister(reg, val) => {
                regs.add_to_register(reg, val);
            }
            OpCodes::SetIndexRegister(addr) => mem.set_index(addr),
            OpCodes::ClearScreen => {
                fb.clear_buffer();
            }
            OpCodes::Display(reg_x, reg_y, height) => {
                let (x, y) = (regs.get(reg_x), regs.get(reg_y));
                // From I to I + N, plot I at VX, VY
                // Simply XOR with existing fb data
                let mut sprite: Vec<u8> = vec![];
                for addr in mem.index.0..mem.index.0 + height {
                    let row = mem.get(addr); // 8 pixels wide because u8
                    sprite.push(row);
                }

                let vf = fb.paint(x, y, sprite) as u8;
                regs.set_register(0xF, vf);
            }
            OpCodes::PushSubroutine(addr) => {
                mem.stack.push(mem.pc.0); // store current instruction to return back
                mem.set_pc(addr);
            }
            OpCodes::PopSubroutine => {
                let addr = mem.stack.pop().unwrap();
                mem.set_pc(addr);
            }
            OpCodes::CopyRegister(vx, vy) => {
                regs.set_register(vx, regs.get(vy));
            }
            OpCodes::Or(vx, vy) => {
                regs.set_register(vx, regs.get(vy) | regs.get(vx));
            }
            OpCodes::And(vx, vy) => {
                regs.set_register(vx, regs.get(vy) & regs.get(vx));
            }
            OpCodes::XOr(vx, vy) => {
                regs.set_register(vx, regs.get(vy) ^ regs.get(vx));
            }
            OpCodes::Add(vx, vy) => {
                let sum = regs.get(vy) + regs.get(vx);
                if sum > 255 {
                    regs.set_register(0xf, 1);
                } else {
                    regs.set_register(0xf, 0);
                }
                regs.set_register(vx, sum);
            }
            OpCodes::SubtractForward(vx, vy) => {
                // todo: carry
                regs.set_register(vx, regs.get(vx) - regs.get(vy));
            }
            OpCodes::SubtractBackward(vx, vy) => {
                // todo: carry
                regs.set_register(vx, regs.get(vy) - regs.get(vx));
            }
            OpCodes::LeftShift(vx, vy) => {
                // optional copy
                let vx_value = regs.get(vy);

                let vf = vx_value | (0b1 << 7);
                let vx_value = vx << 1;

                regs.set_register(vx, vx_value);
                regs.set_register(0xf, vf);
            }
            OpCodes::RightShift(vx, vy) => {
                // optional copy
                let vx_value = regs.get(vy);

                let vf = vx_value & 1;
                let vx_value = vx >> 1;

                regs.set_register(vx, vx_value);
                regs.set_register(0xf, vf);
            }
            OpCodes::Random(vx, nn) => {
                let mut rng = rand::thread_rng();
                let ransuu = rng.gen_range(0..=nn);
                regs.set_register(vx, nn & ransuu);
            }
            OpCodes::JumpWithOffset(addr) => {
                mem.set_pc(addr + regs.get(0) as u16);
            }
            OpCodes::AddToIndex(vx) => {
                // TODO: set VF to 1 if I “overflows” from 0FFF to above 1000 (outside the normal addressing range)
                mem.set_index(mem.index.0 + regs.get(vx) as u16);
            }
            OpCodes::SkipEqualConstant(vx, nn) => {
                if regs.get(vx) == nn {
                    mem.increment_pc();
                }
            }
            OpCodes::SkipNotEqualConstant(vx, nn) => {
                if regs.get(vx) != nn {
                    mem.increment_pc();
                }
            }
            OpCodes::SkipEqualRegister(vx, vy) => {
                if regs.get(vx) == regs.get(vy) {
                    mem.increment_pc();
                }
            }
            OpCodes::SkipNotEqualRegister(vx, vy) => {
                if regs.get(vx) != regs.get(vy) {
                    mem.increment_pc();
                }
            }
            OpCodes::PointChar(vx) => {

            }
            OpCodes::Unimplemented => {}
        }

        fb.sync();

        // simulate OG hardware
        thread::sleep(Duration::from_millis(1_000 / INS_PER_SECOND));
    }
}
