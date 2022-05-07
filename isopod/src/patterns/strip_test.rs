//! Basic demonstrator pattern - points of light zoom from the cenre to the
//! outer edge of the spines.

use crate::common_structs::GpsFix;
use crate::common_structs::ImuReadings;
use crate::common_structs::LedUpdate;
use crate::patterns::Pattern;
use crate::led::{SPINES, LEDS_PER_SPINE};

pub struct StripTest {
    leds: LedUpdate,
    i: usize,
}

impl StripTest {
    pub const NAME: &'static str = "strip_test";
}

impl Pattern for StripTest {
    fn new() -> Box<dyn Pattern> {
        Box::new(Self {
            leds: LedUpdate {
                spines: vec![vec![[0; 3]; LEDS_PER_SPINE]; SPINES],
            },
            i: 0,
        })
    }

    fn step(&mut self, _gps: &Option<GpsFix>, _imu: &ImuReadings) -> &LedUpdate {
        for spine in self.leds.spines.iter_mut() {
            for (idx, led) in spine.iter_mut().enumerate() {
                *led = match (idx + self.i) % 10 {
                    0 => [255, 0, 0],
                    2 => [0, 255, 0],
                    4 => [0, 0, 255],
                    _ => [0, 0, 0],
                };
            }
        }
        self.i = if self.i == 0 { 60 } else { self.i - 1 };
        &self.leds
    }

    fn get_name(&self) -> &'static str {
        Self::NAME
    }
}
