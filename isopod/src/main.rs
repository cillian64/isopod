//! Initialises and starts up worker threads to do the actual work.

use anyhow::Result;
use config::Config;
use lazy_static::lazy_static;
#[cfg(feature = "hardware")]
use rppal::gpio::Gpio;
#[cfg(feature = "hardware")]
use rppal::i2c::I2c;
#[cfg(feature = "hardware")]
use std::fs::File;
#[cfg(feature = "hardware")]
use std::sync::Arc;
use std::thread;
use std::time;

mod circular_buffer;
mod common_structs;
#[cfg(feature = "hardware")]
mod gps;
#[cfg(feature = "hardware")]
mod i2c;
#[cfg(feature = "hardware")]
mod led;
mod motion_sensor;
mod pattern_manager;
mod patterns;
#[cfg(feature = "hardware")]
mod reporter;
mod temperature;
mod control_server;
#[cfg(not(feature = "hardware"))]
use common_structs::ImuReadings;

pub const LEDS_PER_SPINE: usize = 59;
pub const SPINES: usize = 12;

lazy_static! {
    static ref SETTINGS: Config = Config::builder()
        .add_source(config::File::with_name("settings"))
        .build()
        .unwrap();
}

// If bluetooth is enabled then the raspberry pi serial port is
// /dev/ttyS0.  If bluetooth is disabled then /dev/ttyAMA0 is used.
#[cfg(feature = "hardware")]
const SERIAL_PORT: &str = "/dev/ttyS0";

#[cfg(feature = "hardware")]
fn main() -> Result<()> {
    println!("Hello, world!");

    println!("Setting up raw peripherals...");
    println!("Setting up GPIO...");
    let gpio = Gpio::new()?;
    println!("Setting up I2C...");
    let i2c = I2c::new()?;
    println!("Setting up GPS...");
    let file = File::open(SERIAL_PORT)?;
    let reader = std::io::BufReader::new(file);
    println!("Peripherals initialised okay!");

    println!("Setting up peripheral controllers...");
    println!("Setting up I2C peripherals controller...");
    let i2cperiphs = Arc::new(i2c::I2cPeriphs::new(i2c));
    println!("Setting up LED controller...");
    let mut led = led::Led::new(gpio);
    println!("Setting up GPS controller...");
    let gps = Arc::new(gps::Gps::new(reader));
    println!("Peripheral drivers initialised okay!");

    if SETTINGS.get("do_startup_tests")? {
        println!("Doing start-up tests...");
        gps.test()?;
        led.test()?;
        i2cperiphs.test()?;
        println!("Start-up tests look good!");
    } else {
        println!("Skipping start-up tests.");
    }

    println!("Starting worker threads...");
    led.start_thread();
    i2cperiphs.clone().start_thread();
    gps.clone().start_thread();
    let mut reporter = reporter::Reporter::new();

    control_server::start_server();
    println!("Worker threads started.");

    let mut pattern_manager = pattern_manager::PatternManager::new();

    let delay_ms = 1000 / SETTINGS.get::<u64>("fps")?;

    let mut last_report = time::Instant::now();
    let report_interval = SETTINGS.get::<u64>("reporter_interval")?;

    loop {
        // Read latest sensor values
        let gps_fix = gps.get();
        let imu_readings = i2cperiphs.get_imu();
        let battery_readings = i2cperiphs.get_battery();

        // Step pattern and update LEDs
        let led_state = pattern_manager.step(&gps_fix, &imu_readings);
        led.led_update(led_state)?;

        // Send a report if necessary
        let now = time::Instant::now();
        if report_interval > 0 && (now - last_report).as_secs() > report_interval {
            last_report = now;
            // Ignore report errors
            let _res = reporter.send(gps_fix, battery_readings);
        }

        // Sleep until time for the next pattern step
        thread::sleep(time::Duration::from_millis(delay_ms));
    }
}

#[cfg(not(feature = "hardware"))]
fn main() -> Result<()> {
    println!("Hello, world!");
    println!("Simulator mode: skipping setup and self-tests");

    println!("Starting worker threads...");
    // In simulator mode, always enable ws server regardless of config
    let ws = ws_server::WsServer::start_server();
    println!("Worker threads started.");

    let mut pattern_manager = pattern_manager::PatternManager::new();

    let delay_ms = 1000 / SETTINGS.get::<u64>("fps")?;

    loop {
        // Mock up sensor values
        let gps_fix = None;
        let imu_readings = ImuReadings::default();

        // Step pattern and update LEDs
        let led_state = pattern_manager.step(&gps_fix, &imu_readings);
        ws.led_update(led_state)?;

        // Sleep until time for the next pattern step
        thread::sleep(time::Duration::from_millis(delay_ms));
    }
}
