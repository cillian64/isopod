//! Demonstration of a pattern based on accelerometer data: record a moving
//! average of the acceleration, and light up the LEDs when the measured
//! acceleration varies significantly from the moving average.

use crate::common_structs::GpsFix;
use crate::common_structs::ImuReadings;
use crate::common_structs::LedUpdate;
use crate::patterns::Pattern;

const MOVING_AVERAGE_LEN: usize = 30; // Average over half a second
const SHOCK_THRESH: f32 = 3.0; // Shock threshold, in m/s/s

pub struct Shock {
    leds: LedUpdate,

    // A buffer of the last few accelerometer readings seen.  Once it has
    // MOVING_AVERAGE_LEN entries it is written as a circular buffer.
    accel_buffer: Vec<[f32; 3]>,

    // Next position to write in the circular buffer
    i: usize,
}

impl Shock {
    pub const NAME: &'static str = "zoom";
}

impl Pattern for Shock {
    fn new() -> Box<dyn Pattern> {
        Box::new(Self {
            leds: LedUpdate {
                spines: vec![vec![[0; 3]; 60]; 12],
            },
            accel_buffer: vec![],
            i: 0,
        })
    }

    fn step(&mut self, _gps: &Option<GpsFix>, imu: &ImuReadings) -> &LedUpdate {
        // If the accel buffer is not yet full then keep filling it.  Once the
        // accel buffer is full, keep writing it, then calculate a moving
        // average and evaluate the current shock
        let leds_on = if self.accel_buffer.len() < MOVING_AVERAGE_LEN {
            // Just fill buffer, keep LEDs off
            self.accel_buffer.push([imu.xa, imu.ya, imu.za]);
            false
        } else {
            // Calculate the previous moving average
            let mut sum = [0f32; 3];
            for x in &self.accel_buffer {
                sum[0] += x[0];
                sum[1] += x[1];
                sum[2] += x[2];
            }
            let average = [
                sum[0] / (MOVING_AVERAGE_LEN as f32),
                sum[1] / (MOVING_AVERAGE_LEN as f32),
                sum[2] / (MOVING_AVERAGE_LEN as f32),
            ];

            // Update the moving average buffer
            self.accel_buffer[self.i] = [imu.xa, imu.ya, imu.za];
            self.i = (self.i + 1) % MOVING_AVERAGE_LEN;

            // Evaluate the current shock
            let shock = ((imu.xa - average[0]).powi(2)
                + (imu.ya - average[1]).powi(2)
                + (imu.za - average[2]).powi(2))
            .sqrt();
            shock > SHOCK_THRESH
        };

        for spine in 0..12 {
            for led in 0..60 {
                self.leds.spines[spine][led] = if leds_on { [255, 255, 255] } else { [0, 0, 0] };
            }
        }
        &self.leds
    }

    fn get_name(&self) -> &'static str {
        Self::NAME
    }
}
