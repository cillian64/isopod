//! Structs associated with various hardware, but put in here so we can still
//! use them even when the hardware modules are not compiled because we're in
//! simulator mode.

#![allow(unused)]

use crate::patterns::geometry::Vector3d;
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

impl ImuReadings {
    /// Calculate the magnitude of the total acceleration in m/s/s
    pub fn accel_magnitude(&self) -> f32 {
        f32::sqrt(self.xa * self.xa + self.ya * self.ya + self.za * self.za)
    }

    /// Return the total acceleration as a geometry vector
    pub fn accel_vector(&self) -> Vector3d {
        Vector3d::new(self.xa, self.ya, self.za)
    }

    /// Calculate the total magnitude of rotation.  I'm not entirely sure this
    /// makes geometric sense but it's a useful heuristic for how much we're
    /// rotating.
    pub fn gyro_magnitude(&self) -> f32 {
        f32::sqrt(self.xg + self.yg + self.zg)
    }
}

// Implemented to make moving averages a lot neater
impl std::ops::AddAssign<ImuReadings> for ImuReadings {
    fn add_assign(&mut self, other: Self) {
        self.xa += other.xa;
        self.ya += other.ya;
        self.za += other.za;
        self.xg += other.xg;
        self.yg += other.yg;
        self.zg += other.zg;
    }
}

// Implemented to make moving averages a lot neater
impl std::iter::Sum for ImuReadings {
    fn sum<I>(iter: I) -> Self
    where
        I: Iterator<Item = ImuReadings>,
    {
        let mut total = Self::default();
        for item in iter {
            total += item;
        }
        total
    }
}

// Implemented to make moving averages a lot neater
impl std::ops::Div<f32> for ImuReadings {
    type Output = ImuReadings;
    fn div(self, rhs: f32) -> Self {
        ImuReadings {
            xa: self.xa / rhs,
            ya: self.ya / rhs,
            za: self.za / rhs,
            xg: self.xg / rhs,
            yg: self.yg / rhs,
            zg: self.zg / rhs,
        }
    }
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
