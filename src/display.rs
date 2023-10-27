use minifb::{Window, WindowOptions};

const WIDTH: usize = 64;
const HEIGHT: usize = 32;

pub struct FrameBuffer {
    buffer: Vec<u32>,
    pub window: Window,
}

impl FrameBuffer {
    pub fn new() -> Self {
        let buffer = vec![0; WIDTH * HEIGHT];
        let mut window = Window::new(
            "Test - ESC to exit",
            WIDTH,
            HEIGHT,
            WindowOptions::default(),
        )
        .unwrap();
        // Limit to max ~60 fps update rate
        window.limit_update_rate(Some(std::time::Duration::from_micros(16600)));
        Self { buffer, window }
    }

    pub fn clear_buffer(&mut self) {
        self.buffer = vec![0; WIDTH * HEIGHT]
    }

    pub fn sync(&mut self) {
        self.window
            .update_with_buffer(&self.buffer, WIDTH, HEIGHT)
            .unwrap();
    }

    pub fn paint(&mut self, x: u8, y: u8, sprite: Vec<u8>) -> bool {
        let mut vf = false;
        for (i, row) in sprite.iter().enumerate() {
            for j in 0..8 {
                let (nx, ny) = (x + j, y as usize + i);
                let bit = (row >> j) & 1;
                let index = (nx as usize * WIDTH) + (ny * HEIGHT);
                let previous = self.buffer[index];
                self.buffer[index] ^= bit as u32;
                if previous != self.buffer[index] && self.buffer[index] == 0 {
                    vf = true;
                }
            }
        }
        vf
    }
}
