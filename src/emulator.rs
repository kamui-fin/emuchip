use crate::{
    decode::OpCodes, display::FrameBuffer, memory::Memory, registers::Registers, sound::Sound,
};
use minifb::{Key, KeyRepeat};
use rand::Rng;

pub struct Emulator {
    fb: FrameBuffer,
    pub regs: Registers,
    pub mem: Memory,
    pub delay_timer: u8,
    pub sound_timer: u8,
    pub sound: Sound,
}

impl Emulator {
    pub fn init() -> Self {
        let mut mem = Memory::new();

        if let Some(rom) = std::env::args().nth(1) {
            mem.load_rom_by_file(&rom);
        } else {
            panic!("supply a rom file")
        }

        let regs = Registers::new();
        let fb = FrameBuffer::new();
        let sound = Sound::new();

        Self {
            regs,
            mem,
            fb,
            delay_timer: 0,
            sound_timer: 0,
            sound,
        }
    }

    pub fn fetch_decode(&mut self) -> OpCodes {
        let ins = self.mem.next_instruction();
        OpCodes::decode_raw(ins)
    }

    pub fn execute_ins(&mut self, ins: OpCodes) {
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
            OpCodes::LeftShift(vx, _) => {
                let vx_value = self.regs.get(vx);

                let vf = (vx_value >> 7) & 1;
                let vx_value = vx_value << 1;

                self.regs.set_register(vx, vx_value);
                self.regs.set_register(0xf, vf);
            }
            OpCodes::RightShift(vx, _) => {
                let vx_value = self.regs.get(vx);

                let vf = vx_value & 1;
                let vx_value = vx_value >> 1;

                self.regs.set_register(vx, vx_value);
                self.regs.set_register(0xf, vf);
            }
            OpCodes::Random(vx, nn) => {
                let mut rng = rand::thread_rng();
                let ransuu = rng.gen_range(0..=255);
                self.regs.set_register(vx, nn & ransuu);
            }
            OpCodes::JumpWithOffset(addr) => {
                self.mem.set_pc(addr + self.regs.get(0) as u16);
            }
            OpCodes::AddToIndex(vx) => {
                self.mem
                    .set_index(self.mem.index.0 + self.regs.get(vx) as u16);
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
                while digits.len() < 3 {
                    digits.push(0);
                }
                digits.reverse();
                for (i, digit) in digits.iter().enumerate() {
                    self.mem.set(self.mem.index.0 + i as u16, *(digit));
                }
            }
            OpCodes::SkipIfPressed(vx) => {
                self.fb.check_for_keys();
                if self.fb.get_key_status_from_num(self.regs.get(vx)) {
                    self.mem.pc.increment();
                }
            }
            OpCodes::SkipIfNotPressed(vx) => {
                self.fb.check_for_keys();
                if !self.fb.get_key_status_from_num(self.regs.get(vx)) {
                    self.mem.pc.increment();
                }
            }
            OpCodes::CopyDelayToRegister(vx) => self.regs.set_register(vx, self.delay_timer),
            OpCodes::CopyRegisterToDelay(vx) => self.delay_timer = self.regs.get(vx),
            OpCodes::CopyRegisterToSound(vx) => self.sound_timer = self.regs.get(vx),
            OpCodes::GetKey(vx) => {
                let key_pressed = self.fb.wait_for_key();
                self.regs.set_register(vx, key_pressed);
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

    pub fn is_running(&self) -> bool {
        self.fb.window.is_open() && !self.fb.window.is_key_pressed(Key::Escape, KeyRepeat::Yes)
    }

    pub fn sync_timers(&mut self) {
        if self.delay_timer > 0 {
            self.delay_timer -= 1;
        }
        if self.sound_timer > 0 {
            self.sound_timer -= 1;
            self.sound.beep();
        }
    }

    pub fn sync_display(&mut self) {
        self.fb.sync();
    }

    pub fn tick(&mut self) {
        let operation = self.fetch_decode();
        self.execute_ins(operation);
    }

    pub fn sync(&mut self) {
        self.sync_timers();
        self.sync_display();
    }
}
