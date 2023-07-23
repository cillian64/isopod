// I find clippy's style to be less clear
#![allow(clippy::needless_range_loop)]

use crate::common_structs::GpsFix;
use crate::common_structs::ImuReadings;
use crate::common_structs::LedUpdate;
use crate::patterns::Pattern;
use crate::SETTINGS;
use crate::{LEDS_PER_SPINE, SPINES};

use rand::Rng;

use color_space::{Hsv, Rgb};
/// Number of frames per second.  Changing this doesn't actually affect the
/// FPS, that is controlled by frame-skipping, but this value is used to
/// convert average-time-length values into per-frame probabilities
const FPS: f64 = 60.0;

pub struct Rave {
    leds: LedUpdate,
    t: u32,
    rng: rand::rngs::ThreadRng,
    colour: Rgb,
    seg: usize,
}

impl Rave {
    pub const NAME: &'static str = "rave";
}

impl Pattern for Rave {
    fn new() -> Box<dyn Pattern> {
        Box::new(Self {
            leds: LedUpdate::default(),
            rng: rand::thread_rng(),
            t: 0,
            colour: Rgb::new(0.0, 0.0, 0.0),
            seg: 0,
        })
    }

    fn step(&mut self, _gps: &Option<GpsFix>, _imu: &ImuReadings) -> &LedUpdate {
        let donk_rate: u32 = SETTINGS.get("donk_rate").unwrap();
        let donk_len: u32 = SETTINGS.get("donk_len").unwrap();

        let donk: bool = (self.t % donk_rate) < donk_len;

        if self.t % donk_rate == 0 {
            let hsv = Hsv::new(self.rng.gen::<f64>() * 360.0, 1.0, 1.0);
            self.colour = Rgb::from(hsv);
            self.seg = self.rng.gen_range(0..12);
        }

        const FOO: usize = 5;
        if donk {
            for (i, spine) in self.leds.spines.iter_mut().enumerate() {
                if i > self.seg && i < (self.seg + FOO) || (self.seg + FOO > 12 && i < self.seg + FOO - 12) {
                    for led in spine.iter_mut() {
                        *led = [
                            self.colour.r as u8,
                            self.colour.g as u8,
                            self.colour.b as u8,
                        ];
                    }
                } else {
                    for led in spine.iter_mut() {
                        *led = [
                            led[0] / 2,
                            led[1] / 2,
                            led[2] / 2,
                        ];
                    }
                }
            }
        } else {
            for spine in self.leds.spines.iter_mut() {
                for led in spine.iter_mut() {
                        *led = [
                            led[0] / 2,
                            led[1] / 2,
                            led[2] / 2,
                        ];
                }
            }
        }

        self.t += 1;
        &self.leds
    }

    fn get_name(&self) -> &'static str {
        Self::NAME
    }
}
