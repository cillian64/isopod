use crate::gps::GpsFix;
use crate::i2c::ImuReadings;
use crate::led::LedUpdate;

use std::collections::HashMap;
use lazy_static::lazy_static;

pub mod shock;
pub mod zoom;

/// Interface used for creating patterns, either stationary or in motion
pub trait Pattern {
    /// Create a new instance of the pattern.  This is called whenever we
    /// switch from another pattern to this one
    #[allow(clippy::new_ret_no_self)]
    fn new() -> Box<dyn Pattern>
    where
        Self: Sized;

    /// Provide a new set of sensor data to the pattern, and the pattern
    /// should provide an updated LED state.  For efficiency, the pattern
    /// should hold LED state internally and just keep returning a reference
    /// to the same LED state object instead of constructing a new one each
    /// step.  This function is expected to be called 60 times a second.
    /// Patterns can either assume a constant time-step of 1/60th of a second
    /// or use the system clock to acertain the true time step if they want
    /// to.
    /// GPS reading might not be provided if we have never seen a fix (i.e.
    /// because we are trapped indoors or are the wrong way up :-( ).  GPS
    /// data is made optional because most patterns are not expected to use it
    /// anyway.  IMU readings will always be available.
    fn step(&mut self, gps: &Option<GpsFix>, imu: &ImuReadings) -> &LedUpdate;

    /// Get the name of this pattern.  Used for both display and pattern
    /// selection in the configuration file.
    fn get_name(&self) -> &'static str;
}

lazy_static! {
    static ref PATTERNS: HashMap<&'static str, fn() -> Box<dyn Pattern>> = HashMap::from([
        (shock::Shock::NAME, shock::Shock::new as fn() -> Box<dyn Pattern>),
        (zoom::Zoom::NAME, zoom::Zoom::new as fn() -> Box<dyn Pattern>),
    ]);
}

/// Get the constructor for a pattern from its name
pub fn pattern_by_name(name: &str) -> Option<fn() -> Box<dyn Pattern>> {
    PATTERNS
        .iter()
        .find(|(&pattern_name, _cons)| pattern_name == name)
        .map(|(_pattern_name, cons)| *cons)
}