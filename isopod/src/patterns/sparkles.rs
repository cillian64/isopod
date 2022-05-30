//! Sparkles pattern: sparkle random LEDs.

use crate::common_structs::GpsFix;
use crate::common_structs::ImuReadings;
use crate::common_structs::LedUpdate;
use crate::patterns::Pattern;
use crate::LEDS_PER_SPINE;

use rand::Rng;

/// Probability of turning an off LED on
const PROB_ON: f32 = 0.01;

/// Divide the frame rate by this number
const FRAME_SKIP: usize = 3;


pub struct Sparkles {
    leds: LedUpdate,
    rng: rand::rngs::ThreadRng,

    /// Frame counter, used for frame skipping to reduce sparkle rate
    i: usize,
}

impl Sparkles {
    pub const NAME: &'static str = "sparkles";
}

impl Pattern for Sparkles {
    fn new() -> Box<dyn Pattern> {
        Box::new(Self {
            leds: LedUpdate::default(),
            rng: rand::thread_rng(),
            i: 0,
        })
    }

    fn step(&mut self, _gps: &Option<GpsFix>, _imu: &ImuReadings) -> &LedUpdate {
        if self.i % FRAME_SKIP == 0 {
            // Turn all LEDs off, then turn a random selection on
            for spine in self.leds.spines.iter_mut() {
                for led in spine.iter_mut() {
                    *led = [0, 0, 0];
                }
            }

            // Turn all LEDs off except for a random few we turn on for one frame
            for spine in self.leds.spines.iter_mut() {
                for i in 0..LEDS_PER_SPINE {
                    if self.rng.gen::<f32>() < PROB_ON {
                        spine[i] = [255, 255, 255];
                        if i < LEDS_PER_SPINE - 2 {
                            spine[i + 1] = [255, 255, 255];
                        }
                    }
                }
            }
        }

        self.i = (self.i + 1) % FRAME_SKIP;

        &self.leds
    }

    fn get_name(&self) -> &'static str {
        Self::NAME
    }
}
