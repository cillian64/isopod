//! Structs associated with various hardware, but put in here so we can still
//! use them even when the hardware modules are not compiled because we're in
//! simulator mode.

use crate::{LEDS_PER_SPINE, SPINES};
use chrono::{DateTime, TimeZone, Utc};

/// Represents the data captured in a momentary GPS fix
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GpsFix {
    /// Longitude of the fix location in signed decimal degrees
    pub longitude: f64,
    /// Latitude of the fix location in signed decimal degrees
    pub latitude: f64,
    /// Altitude of the fix location in metres
    pub altitude: f32,
    /// The number of satellites in view at the time of the fix
    pub satellites: usize,
    /// The time, in UTC, of the fix
    pub time: DateTime<Utc>,
}

impl std::default::Default for GpsFix {
    fn default() -> Self {
        Self {
            longitude: 0.0, // Welcome to null island
            latitude: 0.0,
            altitude: 0.0,
            satellites: 0,
            time: Utc.ymd(1970, 1, 1).and_hms(0, 0, 0),
        }
    }
}

#[derive(Debug, Clone)]
pub struct LedUpdate {
    pub spines: Vec<Vec<[u8; 3]>>,
}

impl Default for LedUpdate {
    fn default() -> Self {
        Self {
            spines: vec![vec![[0; 3]; LEDS_PER_SPINE]; SPINES],
        }
    }
}

/// Represents the sensor data captured from the IMU at a given instant
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct ImuReadings {
    /// Accelerometer X-axis reading in m/s/s
    pub xa: f32,
    /// Accelerometer Y-axis reading in m/s/s
    pub ya: f32,
    /// Accelerometer Z-axis reading in m/s/s
    pub za: f32,

    /// Gyroscope X-axis reading
    pub xg: f32,
    /// Gyroscope Y-axis reading
    pub yg: f32,
    /// Gyroscope Z-axis reading
    pub zg: f32,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct BatteryReadings {
    /// Pack voltage in volts
    pub voltage: f32,
    /// Pack current in amps.  Negative is discharging, positive is charging
    pub current: f32,
    /// Estimated state-of-charge as a percentage
    pub soc: f32,
}
