use crate::gps::GpsFix;
use crate::i2c::ImuReadings;
use crate::led::LedUpdate;
use crate::patterns::Pattern;

pub struct Zoom {
    leds: LedUpdate,
    i: usize,
}

impl Pattern for Zoom {
    fn new() -> Self {
        Self {
            leds: LedUpdate {
                spines: vec![vec![[0; 3]; 60]; 12],
            },
            i: 0,
        }
    }

    fn step(&mut self, _gps: &Option<GpsFix>, _imu: &ImuReadings) -> &LedUpdate {
        for spine in 0..12 {
            for led in 0..60 {
                self.leds.spines[spine][led] = if (led + self.i) % 10 == 0 {
                    [255, 255, 255]
                } else {
                    [0, 0, 0]
                };
            }
        }
        self.i = if self.i == 0 { 60 } else { self.i - 1 };
        &self.leds
    }
}
