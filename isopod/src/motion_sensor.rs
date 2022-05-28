//! Algorithms for deciding whether or not we're moving

use crate::circular_buffer::CircularBuffer;
use crate::common_structs::ImuReadings;

// Tunable constants for the motion sensing

/// How long to wait after the last detected movement to exit the motion
/// pattern, in frames
pub const MOVEMENT_PATTERN_TIMEOUT: usize = 200;

/// How long to run stationary patterns before going into sleep mode, in
/// frames
const SLEEP_TIMEOUT: usize = 7200;

/// Length of fast moving average filter in frames
const FAST_LPF_LEN: usize = 15;
/// Length of slow moving average filter in frames
const SLOW_LPF_LEN: usize = 600;

/// I hope this doesn't change
const GRAVITY: f32 = 9.81;

/// Threshold to detect fast movement based on accelerometer
const ACCEL_SHOCK_THRESH: f32 = 0.5;

/// Threshold to detect fast movement based on gyro
const GYRO_SHOCK_THRESH: f32 = 1.0;

/// Threshold to detect slow movement
const SLOW_MOVEMENT_THRESH: f32 = 1.0;

/// Algorithms and state for sending slow and fast movements
pub struct MotionSensor {
    /// Circular buffer, used to work out a fast (0.25s) moving average of IMU
    /// readings.
    fast_lpf_buffer: CircularBuffer<ImuReadings>,

    /// Circular buffer used for detecting slow movement over a number of
    /// seconds
    slow_lpf_buffer: CircularBuffer<ImuReadings>,

    /// How long ago did we last see movement.  We track time by the number of
    /// times push() has been called
    samples_since_last_movement: usize,
}

impl MotionSensor {
    /// Build a new motion sensor with empty internal buffers
    pub fn new() -> Self {
        Self {
            fast_lpf_buffer: CircularBuffer::new(FAST_LPF_LEN), // 0.25s
            slow_lpf_buffer: CircularBuffer::new(SLOW_LPF_LEN), // 10s
            samples_since_last_movement: 0,
        }
    }

    /// Push a new set of readings into the internal buffers
    pub fn push(&mut self, imu: ImuReadings) {
        self.fast_lpf_buffer.push(imu);
        if let Some(fast_average) = self.fast_lpf_buffer.mean() {
            self.slow_lpf_buffer.push(fast_average);
        }

        self.samples_since_last_movement += 1;
        if self.detect_fast_movement() || self.detect_slow_movement() {
            self.samples_since_last_movement = 0;
        }
    }

    /// Are we currently experiencing a fast movement or shock?  Will return
    /// false if the fast internal buffer is not yet full
    pub fn detect_fast_movement(&mut self) -> bool {
        if let Some(fast_average) = self.fast_lpf_buffer.mean() {
            let accel_shock =
                f32::abs(fast_average.accel_magnitude() - GRAVITY) > ACCEL_SHOCK_THRESH;
            let gyro_shock = fast_average.gyro_magnitude() > GYRO_SHOCK_THRESH;

            // Each time we see fast movement, reset the creep buffer so that
            // we don't see spurious creeps in stationary periods after
            // movement.
            if accel_shock || gyro_shock {
                self.slow_lpf_buffer.clear();
            }

            accel_shock || gyro_shock
        } else {
            false
        }
    }

    /// Have we moved slowly over the past few seconds?  Will return false if
    /// the slow internal buffer is not yet full
    pub fn detect_slow_movement(&self) -> bool {
        let head = self.slow_lpf_buffer.head();
        let tail = self.slow_lpf_buffer.tail();

        match (head, tail) {
            (Some(head), Some(tail)) => {
                // See if the difference in readings crosses a threshold
                (head.accel_vector() - tail.accel_vector()).magnitude() > SLOW_MOVEMENT_THRESH
            }
            _ => false,
        }
    }

    /// Has it been long enough since movement that the movement pattern should end
    #[allow(unused)]
    pub fn movement_timeout(&self) -> bool {
        self.samples_since_last_movement > MOVEMENT_PATTERN_TIMEOUT
    }

    /// Has it been long enough since movement that we should go to sleep
    pub fn sleep_timeout(&self) -> bool {
        self.samples_since_last_movement > SLEEP_TIMEOUT
    }

    /// Get the average from the fast average buffer
    pub fn get_latest(&self) -> Option<ImuReadings> {
        self.fast_lpf_buffer.mean()
    }
}
