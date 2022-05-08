//! Provides a basic function to read the temperature of the Raspberry Pi

#![allow(unused)]

use std::fs::File;
use std::io::prelude::*;

const TEMPERATURE_PATH: &str = "/sys/class/thermal/thermal_zone0/temp";

/// Get the temperature of the Raspberry Pi in degrees celcius.  May return
/// None if we get an error reading the temperature (for example if this
/// isn't actually a Raspberry Pi or has a weird distro installed).
pub fn get_temperature() -> Option<f32> {
    // An Option is used instead of a Result because running this on PCs where
    // the temperature doesn't exist is an expected use-case.  And even in
    // cases where this fails for odd reasons, we want the error to be
    // non-fatal.

    let mut file = match File::open(TEMPERATURE_PATH) {
        Ok(file) => file,
        Err(_) => return None,
    };
    // println!("Opened file");
    let mut buf = String::new();
    match file.read_to_string(&mut buf) {
        Ok(_) => {}
        Err(_) => return None,
    };
    // println!("Read string {}", buf);
    let millidegrees = match buf.trim().parse::<u32>() {
        Ok(x) => x,
        Err(_) => return None,
    };
    Some(millidegrees as f32 / 1000.0)
}
