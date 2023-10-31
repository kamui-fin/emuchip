use minifb::{Key, Scale, Window, WindowOptions};

const WIDTH: usize = 64;
const HEIGHT: usize = 32;

pub struct FrameBuffer {
    bit_buffer: Vec<u32>,
    pixel_buffer: Vec<u32>,
    pub window: Window,
}

impl FrameBuffer {
    pub fn new() -> Self {
        let mut window = Window::new(
            "emuchip - ESC to exit",
            WIDTH,
            HEIGHT,
            WindowOptions {
                scale: Scale::X16,
                ..WindowOptions::default()
            },
        )
        .unwrap();
        window.set_position(500, 300);
        // Limit to max ~60 fps update rate
        // window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));
        Self {
            bit_buffer: vec![0; WIDTH * HEIGHT],
            pixel_buffer: vec![0; WIDTH * HEIGHT],
            window,
        }
    }

    pub fn clear_buffer(&mut self) {
        self.bit_buffer = vec![0; WIDTH * HEIGHT];
        self.pixel_buffer = vec![0; WIDTH * HEIGHT]
    }

    pub fn sync(&mut self) {
        self.window
            .update_with_buffer(&self.pixel_buffer, WIDTH, HEIGHT)
            .unwrap();
    }

    fn from_u16_rgb(r: u16, g: u16, b: u16) -> u32 {
        let (r, g, b) = (r as u32, g as u32, b as u32);
        (r << 16) | (g << 8) | b
    }

    pub fn paint(&mut self, x: u8, y: u8, sprite: Vec<u8>) -> bool {
        // println!("Painting sprite at ({x}, {y}): {sprite:?}");
        let mut vf = false;
        for (i, row) in sprite.iter().enumerate() {
            for j in 0..8 {
                let (nx, ny) = (x as usize + j, y as usize + i);
                let index = (ny * WIDTH) + nx;
                let bit = (row >> (7 - j)) & 1;
                let previous = self.bit_buffer[index];
                self.bit_buffer[index] ^= bit as u32;
                if previous != self.bit_buffer[index] && self.bit_buffer[index] == 0 {
                    vf = true;
                }

                match self.bit_buffer[index] {
                    0 => {
                        self.pixel_buffer[index] = Self::from_u16_rgb(0, 0, 0);
                    }
                    1 => {
                        self.pixel_buffer[index] = Self::from_u16_rgb(0, 127, 255);
                    }
                    _ => {}
                }
            }
        }
        vf
    }
}

pub const KEYS: [Key; 16] = [
    Key::X,
    Key::Key1,
    Key::Key2,
    Key::Key3,
    Key::Q,
    Key::W,
    Key::E,
    Key::A,
    Key::S,
    Key::D,
    Key::Z,
    Key::C,
    Key::Key4,
    Key::R,
    Key::F,
    Key::V,
];

pub fn key_to_u8(key: Key) -> Option<u8> {
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
