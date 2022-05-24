//! A nice chill "sleep" pattern for extended periods of inactivity

use crate::common_structs::GpsFix;
use crate::common_structs::ImuReadings;
use crate::common_structs::LedUpdate;
use crate::patterns::Pattern;

// Repetition period of the complete pattern, in frames
const PATTERN_PERIOD: usize = 300;

// Period of the sinusoid, in frames
const SINUSOID_PERIOD: usize = 90;

pub struct Sleep {
    leds: LedUpdate,

    // Used for keeping track of time, in frames
    i: usize,
}

impl Sleep {
    pub const NAME: &'static str = "sleep";
}

impl Pattern for Sleep {
    fn new() -> Box<dyn Pattern> {
        Box::new(Self {
            leds: LedUpdate::default(),
            i: 0,
        })
    }

    fn step(&mut self, _gps: &Option<GpsFix>, _imu: &ImuReadings) -> &LedUpdate {
        // The "heartbeat" pattern is defined by a sin^2 function which gives
        // a nice curvy double pulse.  We cut this off after one period (two
        // peaks) and hold black for a while
        let colour: [u8; 3] = if self.i < SINUSOID_PERIOD {
            let t = self.i as f32;
            let omega = std::f32::consts::TAU / (SINUSOID_PERIOD as f32);
            let sin2 = f32::powi(f32::sin(t * omega), 2);
            [f32::round(sin2 * 255.0) as u8, 0, 0]
        } else {
            // Outside of the sinusoidal portion, just black
            [0, 0, 0]
        };

        // Stream LED values out down each spine, while setting the first
        // LED on each spine to the colour defined by the "heartbeat" sin^2
        // function.
        for spine in self.leds.spines.iter_mut() {
            let mut iter = spine.iter_mut().rev().peekable();
            while let Some(led) = iter.next() {
                if let Some(next_led) = iter.peek() {
                    *led = **next_led;
                } else {
                    *led = colour;
                }
            }
        }

        self.i = (self.i + 1) % PATTERN_PERIOD;
        &self.leds
    }

    fn get_name(&self) -> &'static str {
        Self::NAME
    }

    fn is_sleep(&self) -> bool {
        true
    }
}
