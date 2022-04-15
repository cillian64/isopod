//! Controls the attached addressable LEDs using the PWM and GPIO peripherals.

use anyhow::{anyhow, Result};
use rppal::gpio::Gpio;
use std::{thread, time};
use rs_ws281x::{ControllerBuilder, ChannelBuilder, StripType, Controller};
use std::sync::mpsc::{Sender, Receiver, channel};

#[derive(Debug, Clone)]
pub struct LedUpdate {
    pub spines: Vec<Vec<[u8; 3]>>,
}

/// Abstraction for the LED peripheral control, including use of GPIO to
/// switch master power to the LEDs and PWM to output data for the
/// addressable LEDs.
pub struct Led {
    // These will be taken by the thread when it starts
    gpio: Option<Gpio>,
    rx: Option<Receiver<LedUpdate>>,

    // This is left behind for the main thread
    tx: Sender<LedUpdate>,

    thread_started: bool,
}

impl Led {
    pub fn new(gpio: Gpio) -> Self {
        let (tx, rx) = channel();
        Self {
            gpio: Some(gpio),
            rx: Some(rx),
            tx,
            thread_started: false,
        }
    }

    /// Start up a new thread controlling this peripheral.
    pub fn start_thread(self: &mut Self) -> () {
        if !self.thread_started {
            self.thread_started = true;
            let gpio = self.gpio.take().unwrap();
            let rx = self.rx.take().unwrap();
            thread::spawn(move || Self::led_thread(gpio, rx));
        }
    }

    /// The main peripheral control thread
    fn led_thread(_gpio: Gpio, rx: Receiver<LedUpdate>) -> Result<()> {
        // Initialise WS2812b controller here because it's not Send+Sync.
        let mut controller = ControllerBuilder::new()
            .freq(800_000)
            .dma(10)
            .channel(
                0, // Channel Index
                ChannelBuilder::new()
                    .pin(12) // GPIO 12 = PWM0
                    .count(1440) // Number of LEDs
                    .strip_type(StripType::Ws2812)
                    .brightness(64) // default: 255
                    .build(),
            )
            .build()?;

        // Setup a SIGTERM handler to turn off the LEDs before quitting
        let (sigterm_tx, sigterm_rx) = std::sync::mpsc::channel();
        ctrlc::set_handler(move || sigterm_tx.send(()).unwrap())?;

        println!("LED thread running.");

        loop {
            // Exit handler:
            if sigterm_rx.try_recv().is_ok() {
                // Turn off LEDs then quit
                println!("LED thread handling SIGTERM.  Goodbye.");
                Self::set_all_leds(&mut controller, [0, 0, 0, 0]);
                std::process::exit(0);
            }

            // Wait for an LED update
            let led_update = rx.recv().unwrap();

            // Now render the new LED state
            let leds = controller.leds_mut(0);
            for spine in 0..12 {
                for led in 0..60 {
                    // Leds are [B, G, R, W] ordering
                    leds[spine * 120 + led] = [
                        led_update.spines[spine][led][2],
                        led_update.spines[spine][led][1],
                        led_update.spines[spine][led][0],
                        0,
                    ];
                    // Make the loopback LEDs mirrored
                    leds[spine * 120 + 60 + 59 - led] = [
                        led_update.spines[spine][led][2],
                        led_update.spines[spine][led][1],
                        led_update.spines[spine][led][0],
                        0,
                    ];
                }
            }
            controller.render()?;
        }
    }

    /// Perform a quick test of the peripheral.  Must be called before start_thread.
    pub fn test(&self) -> Result<()> {
        if self.thread_started {
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

    pub fn led_update(self: &Self, leds: &LedUpdate) -> Result<()> {
        self.tx.send(leds.clone())?;
        Ok(())
    }
}
