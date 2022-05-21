//! Identify which spine is which

use crate::common_structs::GpsFix;
use crate::common_structs::ImuReadings;
use crate::common_structs::LedUpdate;
use crate::patterns::Pattern;

pub struct IdSpines {
    leds: LedUpdate,

    frame_counter: usize,
}

impl IdSpines {
    pub const NAME: &'static str = "id_spines";
}

impl Pattern for IdSpines {
    fn new() -> Box<dyn Pattern> {
        Box::new(Self {
            leds: LedUpdate::default(),
            frame_counter: 0,
        })
    }

    fn step(&mut self, _gps: &Option<GpsFix>, imu: &ImuReadings) -> &LedUpdate {
        for (spine_idx, spine) in self.leds.spines.iter_mut().enumerate() {
            let number = spine_idx % 4 + 1;
            let colour = match spine_idx / 4 {
                0 => [255, 0, 0],
                1 => [0, 255, 0],
                2 => [0, 0, 255],
                _ => panic!("This shouldn't happen"),
            };

            for (idx, led) in spine.iter_mut().enumerate() {
                // Turn on LEDs in the appropriate colour in groups of N,
                // where N is the spine number % 4 and colours depends on
                // spine number / 3, also pretend spine numbers are 1-based.
                // Leave gaps of 2 black pixels between groups.
                // 0: R  R  R
                // 1: RR  RR  RR
                // 2: RRR  RRR  RRR
                // 3: RRRR  RRRR  RRRR
                // 4: G  G  G
                // 5: GG  GG  GG
                // 6: GGG  GGG  GGG
                // 7: GGGG  GGGG  GGGG
                // 8: B  B  B
                // 9: BB  BB  BB
                // 10: BBB  BBB  BBB
                // 11: BBBB  BBBB  BBBB

                *led = if idx % (number + 2) == 0 || idx % (number + 2) == 1 {
                    [0, 0, 0]
                } else {
                    colour
                };
            }
        }

        // Every second, print accelerometer acceleration readings:
        if self.frame_counter % 60 == 0 {
            println!("Acceleration: {} {} {}", imu.xa, imu.ya, imu.za)
        }
        self.frame_counter += 1;

        &self.leds
    }

    fn get_name(&self) -> &'static str {
        Self::NAME
    }
}
