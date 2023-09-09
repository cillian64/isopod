// I find clippy's style to be less clear
#![allow(clippy::needless_range_loop)]

use crate::common_structs::GpsFix;
use crate::common_structs::ImuReadings;
use crate::common_structs::LedUpdate;
use crate::patterns::Pattern;
use crate::SETTINGS;
use crate::{LEDS_PER_SPINE, SPINES};

pub struct BlueSwirl {
    leds: LedUpdate,
    t: f64,
}

impl BlueSwirl {
    pub const NAME: &'static str = "blue_swirl";
}

impl Pattern for BlueSwirl {
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

                let hue1 = (r_comp + a_comp + self.t).rem_euclid(360.0);
                let colour1 = (hue1 / 360.0 * 2.0 * 3.14159).sin();

                let hue2 = (r_comp + a_comp + self.t + 50.0).rem_euclid(360.0);
                let colour2 = (hue2 / 360.0 * 2.0 * 3.14159).sin();

                *led = [0u8,
                        (colour2 * 100.0) as u8,
                        (colour1 * 250.0) as u8];
            }
        }

        self.t = self.t + speed;
        &self.leds
    }

    fn get_name(&self) -> &'static str {
        Self::NAME
    }
}
