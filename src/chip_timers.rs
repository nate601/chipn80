pub struct ChipTimers {
    pub delay: u8,
    pub sound: u8,
}
impl ChipTimers {
    pub fn new() -> Self {
        Self {
            delay: 0u8,
            sound: 0u8,
        }
    }
    pub fn tick_second(&mut self) {
        if (self.delay > 0) | (self.sound > 0) {
            println!("Delay: {}", self.delay);
            println!("Sound: {}", self.sound);
        }

        self.delay = self.delay.saturating_sub(1);
        self.sound = self.sound.saturating_sub(1);
    }
}
