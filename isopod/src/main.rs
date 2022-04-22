//! Initialises and starts up worker threads to do the actual work.

use anyhow::Result;
use config::Config;
use lazy_static::lazy_static;
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

lazy_static! {
    static ref SETTINGS: Config = Config::builder()
        .add_source(config::File::with_name("settings"))
        .build()
        .unwrap();
}

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
    let ws = if SETTINGS.get("ws_server")? {
        Some(ws_server::WsServer::start_server())
    } else {
        println!("Websocket server disabled.");
        None
    };
    println!("Worker threads started.");

    let mut patterns: Vec<Box<dyn Pattern>> = vec![
        Box::new(patterns::zoom::Zoom::new()),
        Box::new(patterns::shock::Shock::new()),
    ];

    // Select a static pattern based on the config entry
    let desired_pattern: String = SETTINGS.get("static_pattern")?;
    let pattern = patterns.iter_mut().find(|x| x.name() == desired_pattern);

    let pattern = match pattern {
        Some(pattern) => pattern,
        None => {
            println!(
                "WARNING: Pattern '{}' not found, defaulting to first pattern.",
                desired_pattern
            );
            &mut patterns[0]
        }
    };
    println!("Selected pattern: {}", pattern.name());

    let delay_ms = 1000 / SETTINGS.get::<u64>("fps")?;

    let mut last_report = time::Instant::now();
    let report_interval = SETTINGS.get::<u64>("reporter_interval")?;

    loop {
        // Read latest sensor values
        let gps_fix = gps.get();
        let imu_readings = i2cperiphs.get();

        // Step pattern and update LEDs
        let led_state = pattern.step(&gps_fix, &imu_readings);
        led.led_update(led_state)?;
        if let Some(ref ws) = ws {
            ws.led_update(led_state)?;
        }

        // Send a report if necessary
        let now = time::Instant::now();
        if report_interval > 0 && (now - last_report).as_secs() > report_interval {
            last_report = now;
            // Ignore report errors
            println!("Sending report: {:#?}", gps_fix);
            let _res = reporter.send(gps_fix);
        }

        // Sleep until time for the next pattern step
        thread::sleep(time::Duration::from_millis(delay_ms));
    }
}
