#[derive(Debug, Default)]
pub struct Timer {
    pub timer_control: u8,
}

impl Timer {
    pub const fn is_enabled(&self) -> bool {
        self.timer_control & 0b100 == 0b100
    }

    /// Returns the clock speed in Hz
    pub fn clock_speed(&self) -> u32 {
        const M_CYCLES_PER_SECOND: u32 = 0x10_0000; // 1,048,576
        let clock_select = self.timer_control & 0b11;
        M_CYCLES_PER_SECOND
            / match clock_select {
                0b00 => 256, // 256 M-cycles
                0b01 => 4,   // 4 M-cycles
                0b10 => 16,  // 16 M-cycles
                0b11 => 64,  // 64 M-cycles
                _ => unreachable!(),
            }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn test_clock_speed() {
        let mut timer = Timer {
            timer_control: 0b00,
        };
        assert_eq!(timer.clock_speed(), 4096);

        timer.timer_control = 0b01;
        assert_eq!(timer.clock_speed(), 262_144);

        timer.timer_control = 0b10;
        assert_eq!(timer.clock_speed(), 65_536);

        timer.timer_control = 0b11;
        assert_eq!(timer.clock_speed(), 16_384);
    }
}
