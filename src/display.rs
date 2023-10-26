use minifb::{Key, Window, WindowOptions};

const WIDTH: usize = 640;
const HEIGHT: usize = 360;

struct FrameBuffer {
    buffer: Vec<u32>,
    pub window: Window,
}

impl FrameBuffer {
    fn new() -> Self {
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
    fn clear_buffer(&mut self) {
        self.buffer = vec![0; WIDTH * HEIGHT]
    }

    fn sync(&mut self) {
        self.window
            .update_with_buffer(&self.buffer, WIDTH, HEIGHT)
            .unwrap();
    }
}

pub fn start_display() {
    let mut fb = FrameBuffer::new();
    while fb.window.is_open() && !fb.window.is_key_down(Key::Escape) {
        fb.sync();
    }
}
