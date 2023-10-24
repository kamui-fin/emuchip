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

struct Memory {
    // 4k bytes
    // 000 to 1FF is where the interpreter originally resided
    //  Can be left empty besides font
    bytes: [u8; 4096],
}

struct Stack<'a> {
    addresses: &'a [Addr],
}

struct Timer {
    tick: u8,
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

fn main() {
    // main loop (700 CHIP-8 instructions per second)
    // fetch:
    //  read ins @ PC (2 bytes)
    //  increment PC by 2 bytes
    // decode
    //  extract variables
    // execute
    //  run instruction

    display::start_display();
}
