//! Contains the base Pattern trait which all patterns implement.  Imports the
//! module for each pattern and provides a lookup from pattern name to the
//! pattern constructor.

use crate::common_structs::{GpsFix, ImuReadings, LedUpdate};

use lazy_static::lazy_static;
use std::collections::HashMap;

// Patterns
pub mod beans;
pub mod colourfield;
pub mod colourwipes;
pub mod glitch;
pub mod searchlight;
pub mod shock;
pub mod starfield;
pub mod strip_test;
pub mod test_blackout;
pub mod zoom;
pub mod id_spines;

// Other stuff
pub mod geometry;

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
        // Movement patterns
        (
            shock::Shock::NAME,
            shock::Shock::new as fn() -> Box<dyn Pattern>
        ),
        (
            beans::Beans::NAME,
            beans::Beans::new as fn() -> Box<dyn Pattern>
        ),

        // Stationary patterns
        (
            zoom::Zoom::NAME,
            zoom::Zoom::new as fn() -> Box<dyn Pattern>
        ),
        (
            glitch::Glitch::NAME,
            glitch::Glitch::new as fn() -> Box<dyn Pattern>
        ),
        (
            starfield::Starfield::NAME,
            starfield::Starfield::new as fn() -> Box<dyn Pattern>
        ),
        (
            colourfield::Colourfield::NAME,
            colourfield::Colourfield::new as fn() -> Box<dyn Pattern>
        ),
        (
            colourwipes::ColourWipes::NAME,
            colourwipes::ColourWipes::new as fn() -> Box<dyn Pattern>
        ),

        // Test patterns, please ignore
        (
            strip_test::StripTest::NAME,
            strip_test::StripTest::new as fn() -> Box<dyn Pattern>
        ),
        (
            searchlight::Searchlight::NAME,
            searchlight::Searchlight::new as fn() -> Box<dyn Pattern>
        ),
        (
            test_blackout::TestBlackout::NAME,
            test_blackout::TestBlackout::new as fn() -> Box<dyn Pattern>
        ),
        (
            id_spines::IdSpines::NAME,
            id_spines::IdSpines::new as fn() -> Box<dyn Pattern>
        ),
    ]);
}

/// Get the constructor for a pattern from its name
pub fn pattern_by_name(name: &str) -> Option<fn() -> Box<dyn Pattern>> {
    PATTERNS
        .iter()
        .find(|(&pattern_name, _cons)| pattern_name == name)
        .map(|(_pattern_name, cons)| *cons)
}
