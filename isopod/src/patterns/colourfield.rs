//! Like the old Windows screensaver, but with colours.  Based on the "colour
//! rain" battern in my Bitstream project

use crate::common_structs::GpsFix;
use crate::common_structs::ImuReadings;
use crate::common_structs::LedUpdate;
use crate::patterns::Pattern;
use crate::{LEDS_PER_SPINE, SPINES};
use color_space::{Hsv, Rgb};
use rand::Rng;

pub struct Colourfield {
    leds: LedUpdate,
    rng: rand::rngs::ThreadRng,
}

impl Colourfield {
    pub const NAME: &'static str = "colourfield";
}

impl Pattern for Colourfield {
    fn new() -> Box<dyn Pattern> {
        Box::new(Self {
            leds: LedUpdate {
                spines: vec![vec![[0; 3]; LEDS_PER_SPINE]; SPINES],
            },
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
                // Ok, we're making a new star.  A proportion of stars should
                // be coloured, the rest just white
                if self.rng.gen::<f32>() < 0.2 {
                    // Cool, let's make a coloured star
                    let hue = self.rng.gen::<f64>() * 360.0;
                    let saturation = 1.0f64;
                    let value = 1.0f64;
                    let hsv = Hsv::new(hue, saturation, value);
                    let rgb = Rgb::from(hsv);
                    [rgb.r as u8, rgb.g as u8, rgb.b as u8]
                } else {
                    // Ok, boring old white star
                    let intensity: u8 = self.rng.gen_range(85..=150);
                    [intensity, intensity, intensity]
                }
            } else {
                // No star here, officer
                [0, 0, 0]
            };
        }

        &self.leds
    }

    fn get_name(&self) -> &'static str {
        Self::NAME
    }
}
