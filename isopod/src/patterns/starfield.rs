//! Like the old Windows screensaver.

use crate::common_structs::GpsFix;
use crate::common_structs::ImuReadings;
use crate::common_structs::LedUpdate;
use crate::patterns::Pattern;
use rand::Rng;

pub struct Starfield {
    leds: LedUpdate,
    rng: rand::rngs::ThreadRng,
}

impl Starfield {
    pub const NAME: &'static str = "starfield";
}

impl Pattern for Starfield {
    fn new() -> Box<dyn Pattern> {
        Box::new(Self {
            leds: LedUpdate::default(),
            rng: rand::thread_rng(),
        })
    }

    fn step(&mut self, _gps: &Option<GpsFix>, _imu: &ImuReadings) -> &LedUpdate {
        // At each step, each spine has a certain probability of generating a
        // "star" at the root of the spine.  Each star has a probability
        // distribution of brightness.  Based on the basic "rainfall" pattern
        // used in Bitstream.

        for spine in self.leds.spines.iter_mut() {
            // First, shift all the LEDs on this spine down by one
            for led in (1..spine.len()).rev() {
                spine[led] = spine[led - 1];
            }

            // Now decide whether to create a new star at the root
            spine[0] = if self.rng.gen::<f32>() < 0.083 {
                let intensity: u8 = self.rng.gen_range(85..=255);
                [intensity, intensity, intensity]
            } else {
                [0, 0, 0]
            };
        }

        &self.leds
    }

    fn get_name(&self) -> &'static str {
        Self::NAME
    }
}
