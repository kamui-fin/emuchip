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

use std::{fs, process, time::Instant};

use minifb::{Key, KeyRepeat};
use rand::{self, Rng};

use crate::display::FrameBuffer;

mod display;

type TypeAddr = u16; // in reality u12
                     // type TypeRegister = u8; // size of value contained in register AND register label itself

const INS_PER_SECOND: u64 = 700;

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
            stack: Stack::new(),
            font: Font::default(),
        }
    }

    fn set(&mut self, addr: TypeAddr, val: u8) {
        self.bytes[addr as usize] = val;
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

    fn decrement_pc(&mut self) {
        if self.pc.0 == 0 {
            return;
        }
        self.pc.decrement();
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

        /* for i in start_index..start_index + bytes.len() {
            println!("{:03x?} = {:02x?}", i, self.bytes[i]);
        } */

        // load font
        let start_index = 0x50;
        self.bytes[start_index..start_index + self.font.data.len()]
            .copy_from_slice(&self.font.data);
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

    fn set(&mut self, value: u8) {
        self.count = value;
    }

    fn sync(&mut self, last_updated: Instant) -> bool {
        if self.count == 0 {
            return false;
        }
        let elapsed_ms = last_updated.elapsed().as_millis();
        if elapsed_ms > 1_000 / 60 {
            let temp = self.count;

            let num_ticks = elapsed_ms / (1_000 / 60);
            if (self.count as u128) < num_ticks {
                self.count = 0;
            } else {
                self.count -= num_ticks as u8;
            }
            println!("{} -> {}", temp, self.count);
            true
        } else {
            false
        }
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
        let total = self.registers[reg_num as usize].checked_add(value);
        if let Some(total) = total {
            self.registers[reg_num as usize] = total;
        } else {
            // why last 8 bits?
            self.registers[reg_num as usize] =
                ((self.registers[reg_num as usize] as u16 + (value as u16)) & 0b11111111) as u8;
        }
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
    fn decrement(&mut self) {
        self.0 -= 2;
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

struct RawInstruction {
    code: u16,
    i: u8,
}

impl RawInstruction {
    fn new(code: u16) -> Self {
        RawInstruction { code, i: 1 }
    }
    // n is starting digit, m is length
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
        (self.code & (mask << shift_places)) >> shift_places
    }

    // iterator like methods for decoding convenience
    // TODO: better error handling
    fn start_identifier(&mut self) -> u8 {
        if self.i > 1 {
            panic!();
        }
        let id = self.nth_m_digits(self.i, 1);
        self.i += 1;

        id as u8
    }

    fn next_register(&mut self) -> u8 {
        if self.i > 4 {
            panic!();
        }
        let reg = self.nth_m_digits(self.i, 1);
        self.i += 1;
        reg as u8
    }

    fn next_address(&mut self) -> u16 {
        if self.i > 2 {
            panic!();
        }
        let reg = self.nth_m_digits(self.i, 3);
        self.i += 3;
        reg
    }

    fn next_u8(&mut self) -> u8 {
        if self.i > 3 {
            panic!();
        }
        let reg = self.nth_m_digits(self.i, 2);
        self.i += 2;
        reg as u8
    }

    fn next_u4(&mut self) -> u8 {
        self.next_register()
    }
}

impl PartialEq<u16> for RawInstruction {
    fn eq(&self, ins: &u16) -> bool {
        ins.eq(&self.code)
    }
}

#[test]
fn test_bit_manip() {
    assert_eq!(RawInstruction::new(0x4CEE).nth_m_digits(2, 1), 0xC);
    assert_eq!(RawInstruction::new(0x4CEE).nth_m_digits(3, 1), 0xE);
    assert_eq!(RawInstruction::new(0x4CEE).nth_m_digits(1, 1), 0x4);

    assert_eq!(RawInstruction::new(0x4CEE).nth_m_digits(1, 2), 0x4C);
    assert_eq!(RawInstruction::new(0x4CEE).nth_m_digits(2, 2), 0xCE);
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
    Display(u8, u8, u8),

    // 2NNN
    PushSubroutine(TypeAddr),
    // 00EE
    PopSubroutine,

    // 3XNN
    SkipEqualConstant(u8, u8),
    // 4XNN
    SkipNotEqualConstant(u8, u8),
    // 5XY0
    SkipEqualRegister(u8, u8),
    // 9XY0
    SkipNotEqualRegister(u8, u8),

    // 8XY0
    CopyRegister(u8, u8),
    // 8XY1
    Or(u8, u8),
    // 8XY2
    And(u8, u8),
    // 8XY3
    XOr(u8, u8),
    /// 8XY4
    Add(u8, u8),
    // 8XY5
    SubtractForward(u8, u8),
    // 8XY7
    SubtractBackward(u8, u8),
    // 8XYE
    LeftShift(u8, u8),
    // 8XY6
    RightShift(u8, u8),

    // BNNN
    JumpWithOffset(TypeAddr),
    // CXNN
    Random(u8, u8),

    // EX9E
    SkipIfPressed(u8),
    // EXA1
    SkipIfNotPressed(u8),

    // FX07
    CopyDelayToRegister(u8),
    // FX15
    CopyRegisterToDelay(u8),
    // FX18
    CopyRegisterToSound(u8),

    // FX1E
    AddToIndex(u8),
    // FX0A
    GetKey(u8),
    // FX29
    PointChar(u8),
    // FX33
    ToDecimal(u8),

    // FX65
    LoadRegisterFromMemory(u8),
    // FX55
    StoreRegisterToMemory(u8),

    Unimplemented,
}

impl OpCodes {
    fn decode_raw(ins: u16) -> Self {
        let mut raw = RawInstruction::new(ins);

        match raw.start_identifier() {
            0x0 => match ins {
                0x00E0 => Self::ClearScreen,
                0x00EE => Self::PopSubroutine,
                _ => Self::Unimplemented,
            },
            0x1 => Self::Jump(raw.next_address()),
            0x2 => Self::PushSubroutine(raw.next_address()),
            0x3 => Self::SkipEqualConstant(raw.next_register(), raw.next_u8()),
            0x4 => Self::SkipNotEqualConstant(raw.next_register(), raw.next_u8()),
            0x5 => Self::SkipEqualRegister(raw.next_register(), raw.next_register()),
            0x6 => Self::SetRegister(raw.next_register(), raw.next_u8()),
            0x7 => Self::AddToRegister(raw.next_register(), raw.next_u8()),
            0x8 => {
                let (x, y) = (raw.next_register(), raw.next_register());
                let alu_type = raw.next_u4();
                match alu_type {
                    0x0 => Self::CopyRegister(x, y),
                    0x1 => Self::Or(x, y),
                    0x2 => Self::And(x, y),
                    0x3 => Self::XOr(x, y),
                    0x4 => Self::Add(x, y),
                    0x5 => Self::SubtractForward(x, y),
                    0x6 => Self::RightShift(x, y),
                    0x7 => Self::SubtractBackward(x, y),
                    0xE => Self::LeftShift(x, y),
                    _ => Self::Unimplemented,
                }
            }
            0x9 => Self::SkipNotEqualRegister(raw.next_register(), raw.next_register()),
            0xA => Self::SetIndexRegister(raw.next_address()),
            0xB => Self::JumpWithOffset(raw.next_address()),
            0xC => Self::Random(raw.next_register(), raw.next_u8()),
            0xD => Self::Display(raw.next_register(), raw.next_register(), raw.next_u4()),
            0xE => {
                let x = raw.next_register();
                let k_type = raw.next_u8();
                match k_type {
                    0x9E => Self::SkipIfPressed(x),
                    0xA1 => Self::SkipIfNotPressed(x),
                    _ => Self::Unimplemented,
                }
            }
            0xF => {
                let x = raw.next_register();
                let f_type = raw.next_u8();
                match f_type {
                    0x07 => Self::CopyDelayToRegister(x),
                    0x0A => Self::GetKey(x),
                    0x15 => Self::CopyRegisterToDelay(x),
                    0x18 => Self::CopyRegisterToSound(x),
                    0x1E => Self::AddToIndex(x),
                    0x29 => Self::PointChar(x),
                    0x33 => Self::ToDecimal(x),
                    0x55 => Self::StoreRegisterToMemory(x),
                    0x65 => Self::LoadRegisterFromMemory(x),
                    _ => Self::Unimplemented,
                }
            }
            _ => Self::Unimplemented,
        }
    }
}

const KEYS: [Key; 16] = [
    Key::Key0,
    Key::Key1,
    Key::Key2,
    Key::Key3,
    Key::Key4,
    Key::Key5,
    Key::Key6,
    Key::Key7,
    Key::Key8,
    Key::Key9,
    Key::A,
    Key::B,
    Key::C,
    Key::D,
    Key::E,
    Key::F,
];

fn key_to_u8(key: Key) -> Option<u8> {
    match key {
        Key::Key0 => Some(0),
        Key::Key1 => Some(1),
        Key::Key2 => Some(2),
        Key::Key3 => Some(3),
        Key::Key4 => Some(4),
        Key::Key5 => Some(4),
        Key::Key6 => Some(5),
        Key::Key7 => Some(6),
        Key::Key8 => Some(7),
        Key::Key9 => Some(9),
        Key::A => Some(0xA),
        Key::B => Some(0xB),
        Key::C => Some(0xC),
        Key::D => Some(0xD),
        Key::E => Some(0xE),
        Key::F => Some(0xF),
        _ => None,
    }
}

struct Emulator {
    regs: Registers,
    mem: Memory,
    fb: FrameBuffer,
    delay_timer: Timer,
    sound_timer: Timer,
    last_delay: Instant,
    last_sound: Instant,
    last_ins: Instant,
}

impl Emulator {
    fn init() -> Self {
        let regs = Registers::new();
        let mut mem = Memory::new();

        if let Some(rom) = std::env::args().nth(1) {
            mem.load_rom_by_file(&rom);
        } else {
            mem.load_rom(include_bytes!("../rom/1-chip8-logo.ch8"));
        }

        let fb = FrameBuffer::new();

        let delay_timer = Timer::new(0);
        let sound_timer = Timer::new(0);

        let last_delay = Instant::now();
        let last_sound = Instant::now();
        let last_ins = Instant::now();

        Self {
            regs,
            mem,
            fb,
            delay_timer,
            sound_timer,
            last_delay,
            last_sound,
            last_ins,
        }
    }

    fn fetch_decode(&mut self) -> OpCodes {
        let ins = self.mem.next_instruction();
        // println!("Fetching {:#06x}", ins);
        let ins = OpCodes::decode_raw(ins);
        // println!("Decoded to {:#?}", ins);
        ins
    }

    fn execute_ins(&mut self, ins: OpCodes) {
        match ins {
            OpCodes::Jump(addr) => {
                self.mem.set_pc(addr);
            }
            OpCodes::SetRegister(vx, nn) => {
                self.regs.set_register(vx, nn);
            }
            OpCodes::AddToRegister(vx, nn) => {
                self.regs.add_to_register(vx, nn);
            }
            OpCodes::SetIndexRegister(addr) => self.mem.set_index(addr),
            OpCodes::ClearScreen => {
                self.fb.clear_buffer();
                self.fb.sync();
            }
            OpCodes::Display(reg_x, reg_y, height) => {
                let (x, y) = (self.regs.get(reg_x), self.regs.get(reg_y));
                // From I to I + N, plot I at VX, VY
                // Simply XOR with existing fb data
                let mut sprite: Vec<u8> = vec![];
                for addr in self.mem.index.0..self.mem.index.0 + height as u16 {
                    let row = self.mem.get(addr); // 8 pixels wide because u8
                    sprite.push(row);
                }

                let vf = self.fb.paint(x, y, sprite) as u8;
                self.regs.set_register(0xF, vf);
                self.fb.sync();
            }
            OpCodes::PushSubroutine(addr) => {
                self.mem.stack.push(self.mem.pc.0); // store current instruction to return back
                self.mem.set_pc(addr);
            }
            OpCodes::PopSubroutine => {
                let addr = self.mem.stack.pop().unwrap();
                self.mem.set_pc(addr);
            }
            OpCodes::CopyRegister(vx, vy) => {
                self.regs.set_register(vx, self.regs.get(vy));
            }
            OpCodes::Or(vx, vy) => {
                self.regs
                    .set_register(vx, self.regs.get(vy) | self.regs.get(vx));
            }
            OpCodes::And(vx, vy) => {
                self.regs
                    .set_register(vx, self.regs.get(vy) & self.regs.get(vx));
            }
            OpCodes::XOr(vx, vy) => {
                self.regs
                    .set_register(vx, self.regs.get(vy) ^ self.regs.get(vx));
            }
            OpCodes::Add(vx, vy) => {
                let (x, y) = (self.regs.get(vy), self.regs.get(vx));
                let z = x.checked_add(y);
                if let Some(z) = z {
                    self.regs.set_register(vx, z);
                    self.regs.set_register(0xf, 0);
                } else {
                    self.regs
                        .set_register(vx, (((x as u16) + (y as u16)) & 0b11111111) as u8);
                    self.regs.set_register(0xf, 1);
                }
            }
            OpCodes::SubtractForward(vx, vy) => {
                let (x, y) = (self.regs.get(vx), self.regs.get(vy));
                let z = x.checked_sub(y);
                if let Some(z) = z {
                    self.regs.set_register(vx, z);
                    self.regs.set_register(0xf, 1); // no borrow
                } else {
                    self.regs.set_register(vx, x.wrapping_sub(y));
                    self.regs.set_register(0xf, 0); // borrow
                }
            }
            OpCodes::SubtractBackward(vx, vy) => {
                let (x, y) = (self.regs.get(vx), self.regs.get(vy));
                let z = y.checked_sub(x);
                if let Some(z) = z {
                    self.regs.set_register(vx, z);
                    self.regs.set_register(0xf, 1); // no borrow
                } else {
                    self.regs.set_register(vx, y.wrapping_sub(x));
                    self.regs.set_register(0xf, 0); // borrow
                }
            }
            OpCodes::LeftShift(vx, vy) => {
                let vx_value = self.regs.get(vx);

                let vf = (vx_value >> 7) & 1;
                let vx_value = vx_value << 1;

                self.regs.set_register(vx, vx_value);
                self.regs.set_register(0xf, vf);
            }
            OpCodes::RightShift(vx, vy) => {
                let vx_value = self.regs.get(vx);

                let vf = vx_value & 1;
                let vx_value = vx_value >> 1;

                self.regs.set_register(vx, vx_value);
                self.regs.set_register(0xf, vf);
            }
            OpCodes::Random(vx, nn) => {
                let mut rng = rand::thread_rng();
                let ransuu = rng.gen_range(0..=nn);
                self.regs.set_register(vx, nn & ransuu);
            }
            OpCodes::JumpWithOffset(addr) => {
                self.mem.set_pc(addr + self.regs.get(0) as u16);
            }
            OpCodes::AddToIndex(vx) => {
                // Most CHIP-8 interpreters' FX1E instructions do not affect VF
                // with one exception: the CHIP-8 interpreter for the Commodore Amiga sets VF to 1 when there is a range overflow (I+VX>0xFFF)
                // and to 0 when there is not.
                // The only known game that depends on this behavior is Spacefight 2091!, while at least one game, Animal Race, depends on VF not being affected.
                self.mem
                    .set_index(self.mem.index.0 + self.regs.get(vx) as u16);
                // TODO: optional amiga functionality support
            }
            OpCodes::SkipEqualConstant(vx, nn) => {
                if self.regs.get(vx) == nn {
                    self.mem.increment_pc();
                }
            }
            OpCodes::SkipNotEqualConstant(vx, nn) => {
                if self.regs.get(vx) != nn {
                    self.mem.increment_pc();
                }
            }
            OpCodes::SkipEqualRegister(vx, vy) => {
                if self.regs.get(vx) == self.regs.get(vy) {
                    self.mem.increment_pc();
                }
            }
            OpCodes::SkipNotEqualRegister(vx, vy) => {
                if self.regs.get(vx) != self.regs.get(vy) {
                    self.mem.increment_pc();
                }
            }
            OpCodes::PointChar(vx) => {
                let char = self.regs.get(vx);
                let addr = 0x50 + char * 5;
                self.mem.set_index(addr as u16);
            }
            OpCodes::ToDecimal(vx) => {
                let mut in_decimal = self.regs.get(vx);
                let mut digits = vec![];
                while in_decimal != 0 {
                    let left_digit = in_decimal % 10;
                    digits.push(left_digit);
                    in_decimal /= 10;
                }
                digits.reverse();
                for (i, digit) in digits.iter().enumerate() {
                    self.mem.set(self.mem.index.0 + i as u16, *(digit));
                }
            }
            OpCodes::SkipIfPressed(vx) => {
                // TODO: experiment w key repeat
                if self
                    .fb
                    .window
                    .is_key_pressed(KEYS[self.regs.get(vx) as usize], KeyRepeat::Yes)
                {
                    self.mem.pc.increment();
                }
            }
            OpCodes::SkipIfNotPressed(vx) => {
                if !self
                    .fb
                    .window
                    .is_key_pressed(KEYS[self.regs.get(vx) as usize], KeyRepeat::Yes)
                {
                    self.mem.pc.increment();
                }
            }
            OpCodes::CopyDelayToRegister(vx) => self.regs.set_register(vx, self.delay_timer.count),
            OpCodes::CopyRegisterToDelay(vx) => self.delay_timer.set(self.regs.get(vx)),
            OpCodes::CopyRegisterToSound(vx) => self.sound_timer.set(self.regs.get(vx)),
            OpCodes::GetKey(vx) => {
                // TODO: pressed and then released ? or just pressed. original implementation was former
                let pressed = self.fb.window.get_keys();
                println!("{:#?}", pressed);
                if pressed.is_empty() {
                    self.mem.decrement_pc();
                } else if let Some(key) = key_to_u8(pressed[0]) {
                    self.regs.set_register(vx, key);
                }
            }
            OpCodes::LoadRegisterFromMemory(vx) => {
                for reg in 0..=vx {
                    let reg_val = self.mem.get(self.mem.index.0 + reg as u16);
                    self.regs.set_register(reg, reg_val);
                }
            }
            OpCodes::StoreRegisterToMemory(vx) => {
                for reg in 0..=vx {
                    let reg_val = self.regs.get(reg);
                    self.mem.set(self.mem.index.0 + reg as u16, reg_val);
                }
            }
            OpCodes::Unimplemented => {}
        }
    }

    fn is_running(&self) -> bool {
        self.fb.window.is_open() && !self.fb.window.is_key_down(Key::Escape)
    }

    fn sync_timers(&mut self) {
        if self.delay_timer.sync(self.last_delay) {
            self.last_delay = Instant::now();
        }

        if self.sound_timer.sync(self.last_sound) {
            self.last_sound = Instant::now();
        }
    }

    fn can_execute(&mut self) -> bool {
        let result = self.last_ins.elapsed().as_millis() >= (1_000 / INS_PER_SECOND as u128);
        if result {
            self.last_ins = Instant::now();
        }
        result
    }
}

fn main() {
    // main loop (700 CHIP-8 instructions per second)
    let mut emu = Emulator::init();
    while emu.is_running() {
        if emu.can_execute() {
            let operation = emu.fetch_decode();
            emu.execute_ins(operation);
        }
        emu.sync_timers();
    }
}
