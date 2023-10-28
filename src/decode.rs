use crate::memory::TypeAddr;

pub struct RawInstruction {
    code: u16,
    i: u8,
}

impl RawInstruction {
    pub fn new(code: u16) -> Self {
        RawInstruction { code, i: 1 }
    }
    // n is starting digit, m is length
    pub fn nth_m_digits(&self, n: u8, m: u8) -> u16 {
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
    pub fn start_identifier(&mut self) -> u8 {
        if self.i > 1 {
            panic!();
        }
        let id = self.nth_m_digits(self.i, 1);
        self.i += 1;

        id as u8
    }

    pub fn next_register(&mut self) -> u8 {
        if self.i > 4 {
            panic!();
        }
        let reg = self.nth_m_digits(self.i, 1);
        self.i += 1;
        reg as u8
    }

    pub fn next_address(&mut self) -> u16 {
        if self.i > 2 {
            panic!();
        }
        let reg = self.nth_m_digits(self.i, 3);
        self.i += 3;
        reg
    }

    pub fn next_u8(&mut self) -> u8 {
        if self.i > 3 {
            panic!();
        }
        let reg = self.nth_m_digits(self.i, 2);
        self.i += 2;
        reg as u8
    }

    pub fn next_u4(&mut self) -> u8 {
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
pub enum OpCodes {
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
    pub fn decode_raw(ins: u16) -> Self {
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
                /* println!("Original -> {:04x}", ins);
                println!(
                    "Got an 0xF instruction with vx = {:02x} and end = {:02x}",
                    x, f_type
                ); */
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
