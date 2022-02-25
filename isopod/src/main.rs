use std::{thread, time};

use rppal::gpio::Gpio;
use rppal::i2c::I2c;

use linux_embedded_hal as hal;

use anyhow::Result;

use std::fs::File;
use std::io::{self, BufRead};

use nmea::Nmea;

/// Take a parsed NMEA packet from the NMEA library.  Print it if it contains
/// useful info.  Return true if we printed anything, or false if it wasn't
/// full.
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

fn main() -> Result<()> {
    println!("Hello, world!");

    println!("Setting up GPIO...");
    let _gpio = Gpio::new()?;
    println!("Setting up I2C...");
    let mut i2c = I2c::new()?;
    println!("Peripherals initialised okay!");

    println!("Testing I2C and IMU...");
    let mut icm = icm20948::ICMI2C::<_, _, 0x69>::new(&mut i2c)?;
    icm.init(&mut i2c, &mut hal::Delay).unwrap();
    for _ in 0..3 {
        let (xa, ya, za, xg, yg, zg) =
            icm.scale_raw_accel_gyro(icm.get_values_accel_gyro(&mut i2c).unwrap());
        println!(
            "Sensed, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}",
            xa, ya, za, xg, yg, zg
        );
        thread::sleep(time::Duration::from_secs(1));
    }
    println!("I2C and IMU ok!");

    println!("Testing GPS...");
    let file = File::open("/dev/ttyAMA0")?;
    let reader = io::BufReader::new(file);
    let mut nmea = Nmea::new();
    let mut lines_read = 0;
    for line in reader.lines() {
        let line = line?;
        if line.trim().len() == 0 {
            continue;
        }
        match nmea.parse(&line) {
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

    println!("Done.");
    Ok(())
}
