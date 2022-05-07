//! This pattern exists to test the mechanism where power is cut from all the
//! LEDs if no pixels are turned on at all.  It simply flashes one pixel on
//! and off with a period of 2 seconds.  If the mechanism is working correctly
//! then the total power draw of the system should oscillate by over an amp
//! rather than the relatively small power draw of a single LED.

use crate::common_structs::GpsFix;
use crate::common_structs::ImuReadings;
use crate::common_structs::LedUpdate;
use crate::patterns::Pattern;
use crate::led::{SPINES, LEDS_PER_SPINE};

pub struct TestBlackout {
    leds: LedUpdate,
    i: usize,
}

impl TestBlackout {
    pub const NAME: &'static str = "test_blackout";
}

impl Pattern for TestBlackout {
    fn new() -> Box<dyn Pattern> {
        Box::new(Self {
            leds: LedUpdate {
                spines: vec![vec![[0; 3]; LEDS_PER_SPINE]; SPINES],
            },
            i: 0,
        })
    }

    fn step(&mut self, _gps: &Option<GpsFix>, _imu: &ImuReadings) -> &LedUpdate {
        // Counter used for timing
        self.i = (self.i + 1) % 240;

        // Flash one pixel at the beginning of the first spine
        self.leds.spines[0][0] = if self.i < 120 {
            [1, 0, 0]
        } else {
            [0, 0, 0]
        };

        &self.leds
    }

    fn get_name(&self) -> &'static str {
        Self::NAME
    }
}
