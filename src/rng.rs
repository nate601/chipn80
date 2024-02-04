use std::time::SystemTime;
pub(crate) struct RandomNumberGenerator {
    pub(crate) state: u32,
}

impl RandomNumberGenerator {
    pub fn new(state: u32) -> Self {
        Self { state }
    }
    pub fn seed_with_time(&mut self) {
        let time_since_epoch = SystemTime::now().duration_since(SystemTime::UNIX_EPOCH);
        self.state = match time_since_epoch {
            Ok(time) => time.as_millis() as u32,
            Err(_) => {
                println!("System time is before UNIX Epoch!  RNG Seed is static.");
                4u32
            }
        };
    }
    pub fn next(&mut self) -> u8 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.state = x;
        x as u8
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_rng() {
        let mut rng = RandomNumberGenerator::new(0b11110111101000011100110011100001);
        for _ in 0..255 {
            println!("{}", rng.next());
        }
    }
}
