use minifb::Key;

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

pub struct Keyboard {
    keys: [bool; 16],
}

impl Keyboard {
    pub fn new() -> Self {
        Self { keys: [false; 16] }
    }

    pub fn reset(&mut self) {
        self.keys = [false; 16];
    }

    pub fn update_key(&self, key: &Key) {
        match key {
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
        }
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

    pub fn key_to_num(&self, key: Key) -> Result<u8, ()> {
        match key {
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
        }
    }
}
