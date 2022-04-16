//! Polls the GPS periodically to retrieve location data.  Parses NEMA data
//! and stores the useful data.

use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use nmea::Nmea;
use std::fs::File;
use std::io::{self, BufRead};
use std::sync::{Arc, Mutex};
use std::thread;

/// Take a parsed NMEA packet from the NMEA library.  Print it if it contains
/// useful info.  Return true if we printed anything, or false if it wasn't
/// full.
#[allow(dead_code)]
fn print_nmea_packet(packet: &Nmea) -> bool {
    let time = match packet.fix_time {
        Some(time) => time,
        None => return false,
    };
    let longitude = match packet.longitude {
        Some(long) => long,
        None => return false,
    };
    let latitude = match packet.latitude {
        Some(lat) => lat,
        None => return false,
    };
    let altitude = match packet.altitude {
        Some(alt) => alt,
        None => return false,
    };
    let sats = match packet.satellites().len() {
        0 => return false,
        sats => sats,
    };

    println!(
        "{} - {},{} altitude {}.  {} satellites",
        time, latitude, longitude, altitude, sats,
    );

    true
}

/// Take a parsed NMEA packet from the NMEA library.  If it contains a useful
/// fix then return a GpsFix structure, otherwise return None.
fn nmea_to_fix(packet: &Nmea) -> Option<GpsFix> {
    let time = match packet.fix_time {
        Some(time) => time,
        None => return None,
    };
    let date = match packet.fix_date {
        Some(date) => date,
        None => return None,
    };
    let longitude = match packet.longitude {
        Some(long) => long,
        None => return None,
    };
    let latitude = match packet.latitude {
        Some(lat) => lat,
        None => return None,
    };
    let altitude = match packet.altitude {
        Some(alt) => alt,
        None => return None,
    };
    let satellites = match packet.satellites().len() {
        0 => return None,
        sats => sats,
    };

    let naive_date_time = chrono::NaiveDateTime::new(date, time);
    let date_time = DateTime::from_utc(naive_date_time, chrono::Utc);

    Some(GpsFix {
        longitude,
        latitude,
        altitude,
        satellites,
        time: date_time,
    })
}

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

struct GpsInternal {
    thread_started: bool,
    last_fix: Option<GpsFix>,
}

pub struct Gps {
    // Use separate locks because we need to hold the UART lock constantly but
    // want to leave the other lock free so people can read the location.
    internal: Mutex<GpsInternal>,
    reader: Mutex<io::BufReader<File>>,
}

impl Gps {
    pub fn new(reader: io::BufReader<File>) -> Self {
        Self {
            reader: Mutex::new(reader),
            internal: Mutex::new(GpsInternal {
                thread_started: false,
                last_fix: None,
            }),
        }
    }

    #[allow(dead_code)]
    pub fn test(&self) -> Result<()> {
        if self.internal.lock().unwrap().thread_started {
            return Err(anyhow!(
                "Cannot perform test after peripheral thread is running."
            ));
        }

        println!("Testing GPS...");
        let mut reader = self.reader.lock().unwrap();
        let mut nmea = Nmea::new();
        let mut lines_read = 0;
        let mut line_buf = String::new();
        loop {
            line_buf.clear();
            reader.read_line(&mut line_buf)?;
            if line_buf.trim().is_empty() {
                continue;
            }
            match nmea.parse(&line_buf) {
                Ok(_) => {
                    if print_nmea_packet(&nmea) {
                        lines_read += 1;
                        if lines_read >= 5 {
                            break;
                        }
                    }
                }
                Err(_) => continue, // Ignore malformed packets
            };
        }
        println!("GPS ok.");

        Ok(())
    }

    pub fn start_thread(self: Arc<Self>) {
        let thread_started = self.internal.lock().unwrap().thread_started;
        if !thread_started {
            thread::spawn(move || self.gps_thread());
        }
    }

    fn gps_thread(&self) -> ! {
        {
            let mut internal = self.internal.lock().unwrap();
            internal.thread_started = true;
        }

        println!("GPS thread running.");

        let mut reader = self.reader.lock().unwrap();
        let mut nmea = Nmea::new();
        let mut line_buf = String::new();

        loop {
            line_buf.clear();
            // Ignore reader errors, cross fingers that they are temporary
            let _ = reader.read_line(&mut line_buf);
            if line_buf.trim().is_empty() {
                continue;
            }
            match nmea.parse(&line_buf) {
                Ok(_) => match nmea_to_fix(&nmea) {
                    Some(fix) => {
                        let mut internal = self.internal.lock().unwrap();
                        internal.last_fix = Some(fix);
                    }
                    None => {}
                },
                Err(_) => continue, // Ignore malformed packets
            };
        }
    }

    /// If the GPS has ever seen a fix during this execution, then return
    /// details of that fix (which contains the date-time at which the fix
    /// occurred).  Returns None if we have never seen a valid GPS fix.
    pub fn get(&self) -> Option<GpsFix> {
        self.internal.lock().unwrap().last_fix
    }
}
