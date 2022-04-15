//! Controls the attached addressable LEDs using the PWM and GPIO peripherals.

use anyhow::{anyhow, Result};
use rppal::gpio::Gpio;
use std::sync::{Arc, Mutex};
use std::{thread, time};
use rs_ws281x::{ControllerBuilder, ChannelBuilder, StripType, Controller};
use color_space::{Rgb, Hsv};

struct LedInternal {
    _gpio: Gpio,
    thread_started: bool,
    // The colour of each LED on the spines
    spines: Vec<Vec<[u8; 3]>>,
}

#[derive(Debug, Clone)]
pub struct LedUpdate {
    pub spines: Vec<Vec<[u8; 3]>>,
}

/// Abstraction for the LED peripheral control, including use of GPIO to
/// switch master power to the LEDs and PWM to output data for the
/// addressable LEDs.
pub struct Led {
    internal: Mutex<LedInternal>,
}

impl Led {
    pub fn new(gpio: Gpio) -> Self {
        Self {
            internal: Mutex::new(LedInternal {
                _gpio: gpio,
                thread_started: false,
                spines: vec![vec![[0, 0, 0]; 60]; 12],
            }),
        }
    }

    /// Start up a new thread controlling this peripheral.
    pub fn start_thread(self: Arc<Self>) -> () {
        let thread_started = self.internal.lock().unwrap().thread_started;
        if !thread_started {
            thread::spawn(move || self.led_thread());
        }
    }

    /// The main peripheral control thread
    fn led_thread(self: &Self) -> Result<()> {
        {
            let mut internal = self.internal.lock().unwrap();
            internal.thread_started = true;
        }

        // Initialise WS2812b controller here because it's not Send+Sync.
        let mut controller = ControllerBuilder::new()
            .freq(800_000)
            .dma(10)
            .channel(
                0, // Channel Index
                ChannelBuilder::new()
                    .pin(12) // GPIO 12 = PWM0
                    .count(30) // Number of LEDs
                    .strip_type(StripType::Ws2812)
                    .brightness(64) // default: 255
                    .build(),
            )
            .build()?;

        // Setup a SIGTERM handler to turn off the LEDs before quitting
        let (tx, rx) = std::sync::mpsc::channel();
        ctrlc::set_handler(move || tx.send(()).unwrap())?;

        println!("LED thread running.");

        let mut rainbow_offset: f64 = 0.0;
        loop {
            // Exit handler:
            if rx.try_recv().is_ok() {
                // Turn off LEDs then quit
                println!("LED thread handling SIGTERM.  Goodbye.");
                Self::set_all_leds(&mut controller, [0, 0, 0, 0]);
                std::process::exit(0);
            }

            let leds = controller.leds_mut(0);
            // Make a pretty rainbow
            for i in 0..30 {
                let mut hue = (i as f64) / 30.0 * 360.0 + rainbow_offset;
                while hue >= 360.0 {
                    hue -= 360.0;
                }
                let hsv = Hsv::new(hue, 1.0, 1.0);
                let rgb = Rgb::from(hsv);
                leds[i] = Self::rgb_to_led(rgb);
            }
            controller.render()?;

            rainbow_offset += 1.0;
            if rainbow_offset >= 360.0 {
                rainbow_offset = 0.0;
            }

            thread::sleep(time::Duration::from_millis(10));
        }
    }

    /// Perform a quick test of the peripheral.  Must be called before start_thread.
    pub fn test(&self) -> Result<()> {
        let internal = self.internal.lock().unwrap();
        if internal.thread_started {
            return Err(anyhow!(
                "Cannot perform test after peripheral thread is running."
            ));
        }

        println!("Testing WS2812b LED controller");
        let mut controller = ControllerBuilder::new()
            .freq(800_000)
            .dma(10)
            .channel(
                0, // Channel Index
                ChannelBuilder::new()
                    .pin(12) // GPIO 12 = PWM0
                    .count(30) // Number of LEDs
                    .strip_type(StripType::Ws2812)
                    .brightness(255) // default: 255
                    .build(),
            )
            .build()
            .unwrap();

        Self::set_all_leds(&mut controller, [0, 0, 255, 0]); // red
        thread::sleep(time::Duration::from_millis(300));
        Self::set_all_leds(&mut controller, [0, 255, 0, 0]); // green
        thread::sleep(time::Duration::from_millis(300));
        Self::set_all_leds(&mut controller, [255, 0, 0, 0]); // blue

        println!("Finished testing WS2812b LED controller");

        Ok(())
    }

    fn set_all_leds(controller: &mut Controller, argb: [u8; 4]) {
        let leds = controller.leds_mut(0);
        for led in leds {
            *led = argb;
        }
        controller.render().unwrap();
    }

    fn rgb_to_led(rgb: Rgb) -> [u8; 4] {
        [rgb.b as u8,
         rgb.g as u8,
         rgb.r as u8,
         0]
    }

    pub fn set(self: &Self, _leds: &LedUpdate) -> () {
        // TODO
    }
}
