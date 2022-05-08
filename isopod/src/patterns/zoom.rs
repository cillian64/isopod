//! Basic demonstrator pattern - points of light zoom from the cenre to the
//! outer edge of the spines.

use crate::common_structs::GpsFix;
use crate::common_structs::ImuReadings;
use crate::common_structs::LedUpdate;
use crate::patterns::Pattern;
use crate::{LEDS_PER_SPINE, SPINES};

pub struct Zoom {
    leds: LedUpdate,
    i: usize,
}

impl Zoom {
    pub const NAME: &'static str = "zoom";
}

impl Pattern for Zoom {
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
                *led = if (idx + self.i) % 10 == 0 {
                    [255, 255, 255]
                } else {
                    [0, 0, 0]
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
