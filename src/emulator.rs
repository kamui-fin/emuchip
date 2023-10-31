use minifb::{Key, KeyRepeat};
use rand::Rng;
use std::time::Instant;

use crate::{
    decode::OpCodes,
    display::{key_to_u8, FrameBuffer, KEYS},
    memory::Memory,
    registers::Registers,
    sound::Sound,
    timer::Timer,
};

const INS_PER_SECOND: u64 = 3000;
const FPS: u64 = 60;

pub struct Emulator {
    fb: FrameBuffer,
    pub regs: Registers,
    pub mem: Memory,
    pub delay_timer: Timer,
    pub sound_timer: Timer,
    pub last_delay: Instant,
    pub last_sound: Instant,
    pub last_ins: Instant,
    pub last_fb: Instant,
    pub sound: Sound,
}

impl Emulator {
    pub fn init() -> Self {
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
        let last_fb = Instant::now();

        let sound = Sound::new();

        Self {
            regs,
            mem,
            fb,
            delay_timer,
            sound_timer,
            last_delay,
            last_sound,
            last_ins,
            last_fb,
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
                while digits.len() < 3 {
                    digits.push(0);
                }
                digits.reverse();
                for (i, digit) in digits.iter().enumerate() {
                    self.mem.set(self.mem.index.0 + i as u16, *(digit));
                }
            }
            OpCodes::SkipIfPressed(vx) => {
                // TODO: experiment w key repeat
                if self.fb.window.is_key_pressed(
                    crate::display::KEYS[self.regs.get(vx) as usize],
                    KeyRepeat::Yes,
                ) {
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
                let pressed = self.fb.window.get_keys_pressed(KeyRepeat::Yes);
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

    pub fn is_running(&self) -> bool {
        self.fb.window.is_open() && !self.fb.window.is_key_pressed(Key::Escape, KeyRepeat::Yes)
    }

    pub fn sync_timers(&mut self) {
        if self.delay_timer.sync(self.last_delay) {
            self.last_delay = Instant::now();
        }

        if self.sound_timer.sync(self.last_sound) {
            self.sound.beep();
            self.last_sound = Instant::now();
        }
    }

    pub fn sync_display(&mut self) {
        let result = self.last_fb.elapsed().as_millis() >= (1_000 / FPS as u128);
        if result {
            self.fb.sync();
            self.last_fb = Instant::now();
        }
    }

    pub fn can_execute(&mut self) -> bool {
        let result = self.last_ins.elapsed().as_millis()
            >= (1_000 / (INS_PER_SECOND as f64) as u128);
        if result {
            self.last_ins = Instant::now();
        }
        result
    }
}
