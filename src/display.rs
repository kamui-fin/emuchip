use minifb::{Key, Scale, Window, WindowOptions};

const WIDTH: usize = 64;
const HEIGHT: usize = 32;

pub enum VKeys {
    Key1,
    Key2,
    Key3,
    KeyC,
    Key4,
    Key5,
    Key6,
    KeyD,
    Key7,
    Key8,
    Key9,
    KeyE,
    KeyA,
    Key0,
    KeyB,
    KeyF,
}

pub struct FrameBuffer {
    bit_buffer: Vec<u32>,
    pixel_buffer: Vec<u32>,
    pub window: Window,
    keys: [bool; 16],
    should_update: bool,
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
            keys: [false; 16],
            should_update: false,
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

    pub fn check_for_keys(&mut self) {
        self.keys = [false; 16];
        self.window.get_keys().iter().for_each(|key| match key {
            Key::Key1 => self.keys[VKeys::Key1 as usize] = true,
            Key::Key2 => self.keys[VKeys::Key2 as usize] = true,
            Key::Key3 => self.keys[VKeys::Key3 as usize] = true,
            Key::Key4 => self.keys[VKeys::KeyC as usize] = true,
            Key::Q => self.keys[VKeys::Key4 as usize] = true,
            Key::W => self.keys[VKeys::Key5 as usize] = true,
            Key::E => self.keys[VKeys::Key6 as usize] = true,
            Key::R => self.keys[VKeys::KeyD as usize] = true,
            Key::A => self.keys[VKeys::Key7 as usize] = true,
            Key::S => self.keys[VKeys::Key8 as usize] = true,
            Key::D => self.keys[VKeys::Key9 as usize] = true,
            Key::F => self.keys[VKeys::KeyE as usize] = true,
            Key::Z => self.keys[VKeys::KeyA as usize] = true,
            Key::X => self.keys[VKeys::Key0 as usize] = true,
            Key::C => self.keys[VKeys::KeyB as usize] = true,
            Key::V => self.keys[VKeys::KeyF as usize] = true,
            _ => (),
        })
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
            key = match keys[0] {
                Key::Key1 => Ok(0x1),
                Key::Key2 => Ok(0x2),
                Key::Key3 => Ok(0x3),
                Key::Key4 => Ok(0xC),
                Key::Q => Ok(0x4),
                Key::W => Ok(0x5),
                Key::E => Ok(0x6),
                Key::R => Ok(0xD),
                Key::A => Ok(0x7),
                Key::S => Ok(0x8),
                Key::D => Ok(0x9),
                Key::F => Ok(0xE),
                Key::Z => Ok(0xA),
                Key::X => Ok(0x0),
                Key::C => Ok(0xB),
                Key::V => Ok(0xF),
                Key::Escape => std::process::exit(0),
                _ => Err(()),
            };
            self.sync();
            keys = self.window.get_keys();
        }
        key.unwrap()
    }

    pub fn get_key_status_from_vkey(&mut self, key: VKeys) -> bool {
        self.keys[key as usize]
    }

    pub fn get_key_status_from_num(&mut self, n: u8) -> bool {
        match n {
            0x1 => self.get_key_status_from_vkey(VKeys::Key1),
            0x2 => self.get_key_status_from_vkey(VKeys::Key2),
            0x3 => self.get_key_status_from_vkey(VKeys::Key3),
            0xC => self.get_key_status_from_vkey(VKeys::KeyC),
            0x4 => self.get_key_status_from_vkey(VKeys::Key4),
            0x5 => self.get_key_status_from_vkey(VKeys::Key5),
            0x6 => self.get_key_status_from_vkey(VKeys::Key6),
            0xD => self.get_key_status_from_vkey(VKeys::KeyD),
            0x7 => self.get_key_status_from_vkey(VKeys::Key7),
            0x8 => self.get_key_status_from_vkey(VKeys::Key8),
            0x9 => self.get_key_status_from_vkey(VKeys::Key9),
            0xE => self.get_key_status_from_vkey(VKeys::KeyE),
            0xA => self.get_key_status_from_vkey(VKeys::KeyA),
            0x0 => self.get_key_status_from_vkey(VKeys::Key0),
            0xB => self.get_key_status_from_vkey(VKeys::KeyB),
            0xF => self.get_key_status_from_vkey(VKeys::KeyF),
            _ => panic!("unable to parse key number"),
        }
    }

    pub fn paint(&mut self, x: u8, y: u8, sprite: Vec<u8>) -> bool {
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
        self.should_update = true;
        vf
    }
}
