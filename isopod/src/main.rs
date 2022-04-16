//! Initialises and starts up worker threads to do the actual work.

use anyhow::Result;
use rppal::gpio::Gpio;
use rppal::i2c::I2c;
use std::fs::File;
use std::sync::Arc;
use std::thread;
use std::time;

mod gps;
mod i2c;
mod led;
mod patterns;
mod reporter;
mod ws_server;

use patterns::Pattern;

// If bluetooth is enabled then the raspberry pi serial port is
// /dev/ttyS0.  If bluetooth is disabled then /dev/ttyAMA0 is used.
const SERIAL_PORT: &str = "/dev/ttyS0";

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

    // println!("Doing start-up tests...");
    // gps.test()?;
    // led.test()?;
    // i2cperiphs.test()?;
    // println!("Start-up tests look good!");

    println!("Starting worker threads...");
    led.start_thread();
    i2cperiphs.clone().start_thread();
    gps.clone().start_thread();
    let mut _reporter = reporter::Reporter::new();
    let ws = ws_server::WsServer::start_server();
    println!("Worker threads started.");

    // Main application loop
    //loop {
    //    // Every few seconds, send a report
    //    thread::sleep(time::Duration::from_secs(10));
    //    let fix = gps.get();
    //    reporter.send(fix)?;
    //}

    let mut zoomer = patterns::zoom::Zoom::new();
    loop {
        let gps_fix = gps.get();
        let imu_readings = i2cperiphs.get();

        let led_state = zoomer.step(&gps_fix, &imu_readings);
        led.led_update(led_state)?;
        ws.led_update(led_state)?;
        thread::sleep(time::Duration::from_millis(1000 / 60));
    }
}
