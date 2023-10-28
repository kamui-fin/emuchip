use std::{fs, process};

use crate::registers::{IndexRegister, ProgramCounter};

pub type TypeAddr = u16; // in reality u12
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

pub struct Memory {
    // 4k bytes
    // font data stored from 050 -> 09F (000 -> 04F is empty by convention)
    bytes: [u8; 4096],
    pub pc: ProgramCounter,
    pub index: IndexRegister,
    font: Font,
    pub stack: Stack,
}

impl Memory {
    pub fn new() -> Self {
        Self {
            bytes: [0; 4096],
            pc: ProgramCounter(0x200, 0),
            index: IndexRegister(0x0),
            stack: Stack::new(),
            font: Font::default(),
        }
    }

    pub fn set(&mut self, addr: TypeAddr, val: u8) {
        self.bytes[addr as usize] = val;
    }

    pub fn get(&self, addr: TypeAddr) -> u8 {
        self.bytes[addr as usize]
    }

    pub fn increment_pc(&mut self) {
        let result = self.pc.increment();
        if !result {
            process::exit(0);
        }
    }

    pub fn decrement_pc(&mut self) {
        if self.pc.0 == 0 {
            return;
        }
        self.pc.decrement();
    }

    pub fn next_instruction(&mut self) -> u16 {
        let (l, r) = (
            self.bytes[self.pc.0 as usize],
            self.bytes[(self.pc.0 + 1) as usize],
        );
        self.increment_pc();
        ((l as u16) << 8) | r as u16
    }

    pub fn set_pc(&mut self, addr: TypeAddr) {
        self.pc.set_addr(addr);
    }

    pub fn set_index(&mut self, addr: TypeAddr) {
        self.index.set_addr(addr);
    }

    // loads program instructions starting at address 0x09F
    pub fn load_rom(&mut self, bytes: &[u8]) {
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

    pub fn load_rom_by_file(&mut self, path: &str) {
        let program = fs::read(path).unwrap();
        self.load_rom(program.as_slice());
    }
}

pub struct Stack {
    addresses: Vec<TypeAddr>,
}

impl Stack {
    pub fn new() -> Self {
        Self { addresses: vec![] }
    }

    pub fn push(&mut self, addr: TypeAddr) {
        self.addresses.push(addr)
    }

    pub fn pop(&mut self) -> Option<TypeAddr> {
        self.addresses.pop()
    }
}
