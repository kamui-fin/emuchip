use minifb::{Key, Scale, Window, WindowOptions};

use crate::keyboard::Keyboard;

const WIDTH: usize = 64;
const HEIGHT: usize = 32;

pub struct FrameBuffer {
    bit_buffer: Vec<u32>,
    pixel_buffer: Vec<u32>,
    pub window: Window,
    should_update: bool,
    pub keyboard: Keyboard,
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
        window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));
        Self {
            bit_buffer: vec![0; WIDTH * HEIGHT],
            pixel_buffer: vec![0; WIDTH * HEIGHT],
            window,
            should_update: false,
            keyboard: Keyboard::new(),
        }
    }

    pub fn clear_buffer(&mut self) {
        self.bit_buffer = vec![0; WIDTH * HEIGHT];
        self.pixel_buffer = vec![0; WIDTH * HEIGHT];
        self.should_update = true;
    }

    pub fn sync(&mut self) {
        if self.should_update {
            self.window
                .update_with_buffer(&self.pixel_buffer, WIDTH, HEIGHT)
                .unwrap();
        }
    }

    fn from_u16_rgb(r: u16, g: u16, b: u16) -> u32 {
        let (r, g, b) = (r as u32, g as u32, b as u32);
        (r << 16) | (g << 8) | b
    }

    pub fn paint(&mut self, x: u8, y: u8, sprite: Vec<u8>) -> bool {
        let (x, y) = (x as usize % (WIDTH), y as usize % (HEIGHT));
        let mut vf = false;
        for (i, row) in sprite.iter().enumerate() {
            for j in 0..8 {
                let (nx, ny) = (x as usize + j, y as usize + i);
                let index = (ny * WIDTH) + nx;
                let bit = (row >> (7 - j)) & 1;
                if index >= self.bit_buffer.len() {
                    continue; // should not wrap, cut-off instead
                }
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
        self.should_update = true;
        vf
    }

    pub fn check_for_keys(&mut self) {
        self.keyboard.reset();
        self.window
            .get_keys()
            .iter()
            .for_each(|key| self.keyboard.update_key(key));
    }

    pub fn wait_for_key(&mut self) -> u8 {
        self.sync();
        let mut key: Result<u8, ()> = Err(());
        let mut keys = self.window.get_keys();
        while key == Err(()) {
            while keys.is_empty() {
                self.sync();
                keys = self.window.get_keys();
            }
            key = self.keyboard.key_to_num(keys[0]);
            self.sync();

            keys = self.window.get_keys();
        }
        key.unwrap()
    }
}
