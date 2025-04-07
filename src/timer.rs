#[derive(Debug, Default, Clone, Copy)]
pub struct Timer {
    pub control: u8,
    /// DIV
    pub divider: u8,
    /// TIMA
    pub counter: u8,
    /// TMA
    pub modulo: u8,

    /// Internal counter to track how many cycles its been since the last divider increment
    divider_counter: u8,
    /// Internal counter to track how many cycles its been since timer was last incremented
    counter_counter: u16,
}

impl Timer {
    /// Returns if interrupt should be triggered
    pub fn step(&mut self, cycles: u8) -> bool {
        let (divider, div_counter) = self.increment_div(cycles);
        let mut did_overflow = false;

        self.divider = divider;
        self.divider_counter = div_counter;
        if self.is_enabled() {
            let cycle_target = self.cycle_speed() * 4;
            self.counter_counter += u16::from(cycles);

            // the longest an instruction can take is at least 20 cycles, and the timer can step as
            // quickly as every 16 cycles, so we need to loop here
            while self.counter_counter >= cycle_target {
                self.counter_counter -= cycle_target;
                let (counter, overflow) = self.counter.overflowing_add(1);

                // FIXME: If a TMA write is executed on the same M-cycle as the content of TMA
                // is transferred to TIMA due to a timer overflow, the old value is transferred
                // to TIMA.

                self.counter = if overflow { self.modulo } else { counter };
                if overflow {
                    did_overflow = true;
                }
            }
        }

        did_overflow
    }

    pub const fn is_enabled(self) -> bool {
        self.control & 0b100 == 0b100
    }

    /// Returns the clock speed in M-states
    pub const fn cycle_speed(self) -> u16 {
        let clock_select = self.control & 0b11;
        match clock_select {
            0b00 => 256, // 256 M-states
            0b01 => 4,   // 4 M-states
            0b10 => 16,  // 16 M-states
            0b11 => 64,  // 64 M-states
            _ => unreachable!(),
        }
    }

    const fn increment_div(self, cycles: u8) -> (u8, u8) {
        // divider is incremented at a rate of 16,384Hz - every 256 T-states
        let (counter, overflow) = self.divider_counter.overflowing_add(cycles);
        let divider = if overflow {
            self.divider.wrapping_add(1)
        } else {
            0
        };
        (divider, counter)
    }
}
