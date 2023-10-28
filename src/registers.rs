use crate::memory::TypeAddr;

pub struct Registers {
    registers: [u8; 16],
}

impl Registers {
    pub fn new() -> Self {
        Self { registers: [0; 16] }
    }

    pub fn set_register(&mut self, reg_num: u8, value: u8) {
        self.registers[reg_num as usize] = value;
    }

    pub fn add_to_register(&mut self, reg_num: u8, value: u8) {
        let total = self.registers[reg_num as usize].checked_add(value);
        if let Some(total) = total {
            self.registers[reg_num as usize] = total;
        } else {
            // why last 8 bits?
            self.registers[reg_num as usize] =
                ((self.registers[reg_num as usize] as u16 + (value as u16)) & 0b11111111) as u8;
        }
    }

    pub fn get(&self, reg_num: u8) -> u8 {
        self.registers[reg_num as usize]
    }
}

// Special registers
#[derive(Debug)]
pub struct ProgramCounter(pub TypeAddr, pub TypeAddr);

impl ProgramCounter {
    pub fn increment(&mut self) -> bool {
        self.0 += 2;
        self.0 <= self.1
    }
    pub fn decrement(&mut self) {
        self.0 -= 2;
    }

    pub fn set_end(&mut self, len: usize) {
        self.1 = self.0 + (len as u16);
    }

    pub fn set_addr(&mut self, addr: TypeAddr) {
        self.0 = addr;
    }
}

pub struct IndexRegister(pub TypeAddr);

impl IndexRegister {
    pub fn set_addr(&mut self, addr: TypeAddr) {
        self.0 = addr;
    }
}
