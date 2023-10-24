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

mod display;

type Addr = u16; // in reality u12

/* Low level emulation mappers */

struct Memory {
    // 4k bytes
    // font data stored from 050 -> 09F (000 -> 04F is empty by convention)
    bytes: [u8; 4096],
}

struct Stack<'a> {
    addresses: &'a [Addr],
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
    V0(Addr),
    V1(Addr),
    V2(Addr),
    V3(Addr),
    V4(Addr),
    V5(Addr),
    V6(Addr),
    V7(Addr),
    V8(Addr),
    V9(Addr),
    VA(Addr),
    VB(Addr),
    VC(Addr),
    VD(Addr),
    VE(Addr),
    VF(Addr),

    // Special registers
    PC(Addr),
    I(Addr),
}

enum OpCodes {
    // 00E0
    // turn all pixels to 0
    ClearScreen,
    // 1NNN
    // set PC to address NNN, "jump" to memory location
    Jump,
    // 6XNN
    // set register VX to value NN
    SetRegister,
    // 7XNN
    // add value NN to VX
    AddToRegister,
    // ANNN
    // set index register I to address NNNN
    SetIndexRegister,
    // DXYN (hardest)
    // draw an N pixel tall sprite starting at I
    // at Coordinates (VX, VY)
    // XOR pixels on screen using sprite data
    // if pixels on screen were switched OFF: VF set to 1
    Display,
}

fn main() {
    // main loop (700 CHIP-8 instructions per second)
    // fetch:
    //  read ins @ PC (2 bytes)
    //  increment PC by 2 bytes
    // decode
    //  extract variables
    // execute
    //  run instruction

    // display::start_display();
}
