use std::time::Instant;

const TIMER_DEC_PER_SECOND: u64 = 60;

#[derive(Debug)]
pub struct Timer {
    pub count: u8,
}

impl Timer {
    pub fn new(init_count: u8) -> Self {
        Self { count: init_count }
    }

    pub fn set(&mut self, value: u8) {
        self.count = value;
    }

    pub fn sync(&mut self, last_updated: Instant) -> bool {
        if self.count == 0 {
            return false;
        }
        let elapsed_ms = last_updated.elapsed().as_millis();
        if elapsed_ms >= 1_000 / (TIMER_DEC_PER_SECOND as f64) as u128 {
            // past deadline
            self.count -= 1;
            true
        } else {
            false
        }
    }
}
