//! "glitch" pattern: randomly illuminate segments of the spines in random
//! bright colours

// I find clippy's style to be less clear
#![allow(clippy::needless_range_loop)]

use crate::common_structs::GpsFix;
use crate::common_structs::ImuReadings;
use crate::common_structs::LedUpdate;
use crate::patterns::Pattern;
use crate::SETTINGS;
use crate::{LEDS_PER_SPINE, SPINES};

use color_space::{Hsv, Rgb};

pub struct RainbowSwirl {
    leds: LedUpdate,
    t: f64,
}

impl RainbowSwirl {
    pub const NAME: &'static str = "rainbow_swirl";
}

impl Pattern for RainbowSwirl {
    fn new() -> Box<dyn Pattern> {
        Box::new(Self {
            leds: LedUpdate::default(),
            t: 0.0,
        })
    }

    fn step(&mut self, _gps: &Option<GpsFix>, _imu: &ImuReadings) -> &LedUpdate {
        let radial_smear: f64 = SETTINGS.get("rainbow_swirl_radial_smear").unwrap();
        let speed: f64 = SETTINGS.get("rainbow_swirl_speed").unwrap();

        // Turn all LEDs off
        for (theta, spine) in self.leds.spines.iter_mut().enumerate() {
            for (r, led) in spine.iter_mut().enumerate() {
                let rratio = r as f64 / LEDS_PER_SPINE as f64;
                let r_comp = rratio * 360.0 * radial_smear;
                let tratio = theta as f64 / SPINES as f64;
                let a_comp = tratio * 360.0;

                let hue = (r_comp + a_comp + self.t).rem_euclid(360.0);

                let hsv = Hsv::new(hue, 1.0, 1.0);
                let rgb = Rgb::from(hsv);
                *led = [rgb.r as u8, rgb.g as u8, rgb.b as u8];
            }
        }

        self.t = self.t + speed;
        &self.leds
    }

    fn get_name(&self) -> &'static str {
        Self::NAME
    }
}
