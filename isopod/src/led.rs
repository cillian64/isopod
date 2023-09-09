//! Controls the attached addressable LEDs using the PWM and GPIO peripherals.

use crate::common_structs::LedUpdate;
use crate::SETTINGS;
use crate::control_server::CONTROLS;
use crate::{LEDS_PER_SPINE, SPINES};
use anyhow::{anyhow, Result};
use rppal::gpio::Gpio;
use rs_ws281x::{ChannelBuilder, Controller, ControllerBuilder, StripType};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::{thread, time};

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

/// Make a new controller
fn get_controller() -> Result<Controller> {
    let brightness: u8 = SETTINGS.get("led_brightness")?;
    Ok(ControllerBuilder::new()
        .freq(800_000)
        .dma(10)
        .channel(
            0, // Channel Index
            ChannelBuilder::new()
                .pin(12) // GPIO 12 = PWM0
                .count((SPINES * LEDS_PER_SPINE / 2) as i32) // Number of LEDs
                .strip_type(StripType::Ws2812)
                .brightness(brightness)
                .build(),
        )
        .channel(
            1, // Channel Index
            ChannelBuilder::new()
                .pin(13) // GPIO 13 = PWM1
                .count((SPINES * LEDS_PER_SPINE / 2) as i32) // Number of LEDs
                .strip_type(StripType::Ws2812)
                .brightness(brightness)
                .build(),
        )
        .build()?)
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
    pub fn start_thread(&mut self) {
        if !self.thread_started {
            self.thread_started = true;
            let gpio = self.gpio.take().unwrap();
            let rx = self.rx.take().unwrap();
            thread::Builder::new()
                .name("ISOPOD LED".into())
                .spawn(move || Self::led_thread(gpio, rx))
                .unwrap();
        }
    }

    /// The main peripheral control thread
    fn led_thread(gpio: Gpio, rx: Receiver<LedUpdate>) -> Result<()> {
        // When LEDs are disabled we drop the controller so as to leave the
        // PWM lines idle and stop the LEDs being semi-powered through their
        // data->ground diode.
        let mut controller: Option<Controller> = None;

        // Setup a SIGTERM handler to turn off the LEDs before quitting
        let (sigterm_tx, sigterm_rx) = std::sync::mpsc::channel();
        ctrlc::set_handler(move || sigterm_tx.send(()).unwrap())?;

        // Setup the LED enable pin and default LEDs to off
        let mut led_enable_pin = gpio.get(5)?.into_output();
        led_enable_pin.set_low();

        let map = Self::get_led_mapping()?;

        println!("LED thread running.");

        // Count how many frames we see in a row where all LEDs are disabled.
        let mut black_frames: usize = 0;

        loop {
            // Exit handler:
            if sigterm_rx.try_recv().is_ok() {
                // Turn off LEDs then quit
                println!("LED thread handling SIGTERM.  Goodbye.");
                if let Some(ref mut controller) = controller {
                    Self::set_all_leds(controller, [0, 0, 0, 0]);
                    led_enable_pin.set_low();
                }

                // Destroy the controller, then park the PWM pins at +VCC.
                // After the level shifter this will force the data pins to
                // both be 5V DC preventing parasitic powering of the LEDs.
                controller.take();
                let mut pwm0 = gpio.get(12)?.into_output();
                let mut pwm1 = gpio.get(13)?.into_output();
                pwm0.set_reset_on_drop(false);
                pwm1.set_reset_on_drop(false);
                pwm0.set_high();
                pwm1.set_high();
                std::process::exit(0);
            }

            // Wait for an LED update.  If multiple messages are waiting then
            // receive all of them and discard all but the last.  This means
            // if we are too slow then we will drop packets rather than
            // falling behind and letting the buffer grow indefinitely.
            let mut led_update = rx.recv().unwrap();
            while let Ok(further_update) = rx.try_recv() {
                // Only print warnings if LEDs are actually enabled.  Our way
                // of disabling the LEDs causes some spurious frame-drops.
                if controller.is_some() {
                    println!("Warning: LED update packet dropped!");
                }
                led_update = further_update;
            }

            // Decide whether LEDs should be enabled: cut power after a number
            // of frames in a row where all LEDs are off.
            if led_update
                .spines
                .iter()
                .all(|leds| leds.iter().all(|led| led == &[0, 0, 0]))
            {
                // All pixels are off
                if black_frames < 3 {
                    black_frames += 1;
                }
            } else {
                // At least one pixel is not totally off
                black_frames = 0;
            }
            let leds_enabled = black_frames < 3;

            // If LEDs are newly enabled, bring up the controller and set the
            // LED enable pin.  If they are newly disabled, destroy the
            // controller and force the PWM pins high so the LEDs can't ground
            // via the data pin.

            if leds_enabled && controller.is_none() {
                // LEDs newly enabled
                controller = Some(get_controller()?);
                led_enable_pin.set_high();
            } else if !leds_enabled && controller.is_some() {
                // LEDs newly disabled

                // This "if let" is always true but it's neater than the alternative
                if let Some(ref mut controller) = controller {
                    // If we don't do this and just cut power then the LEDs
                    // fade out over a fraction of a second and go orangey red
                    // colours.  This should give a snappy clean turn-off.
                    Self::set_all_leds(controller, [0, 0, 0, 0]);
                    controller.render()?;
                }
                // Destroy the controller so things don't get weird when we
                // try to take its pins as GPIOs.
                controller.take();
                led_enable_pin.set_low();

                // Make the PWM pins constant-high.  After the level shifter
                // this will force the data pins to both be 5V DC.
                let mut pwm0 = gpio.get(12)?.into_output();
                let mut pwm1 = gpio.get(13)?.into_output();
                pwm0.set_reset_on_drop(false);
                pwm1.set_reset_on_drop(false);
                pwm0.set_high();
                pwm1.set_high();
            }

            // Now render the new LED state (if LEDs are enabled).  Apply
            // position mapping (it won't affect the software visualiser whose
            // data doesn't come through this module).
            if let Some(ref mut controller) = controller {
                // Work out what if any power limiting scaling is needed
                let mut power_scale = Self::get_power_limit_scaling(&led_update);

                // Apply scaling from control panle
                let brightness = CONTROLS.read().unwrap().brightness;
                if brightness < 100 {
                    power_scale = Some((brightness as f32) / 100.0);
                }

                let leds = controller.leds_mut(0);
                // spine_hard represents a physical LED connector on the PCB
                for spine_hard in 0..(SPINES / 2) {
                    // Figure out which logical spine this connector maps to
                    let spine_logical = map[spine_hard] - 1;
                    for led in 0..LEDS_PER_SPINE {
                        let (r, g, b) = (
                            led_update.spines[spine_logical][led][0],
                            led_update.spines[spine_logical][led][1],
                            led_update.spines[spine_logical][led][2],
                        );

                        // Apply power scaling if necessary
                        let (r, g, b) = (
                            Self::scale_val(r, power_scale),
                            Self::scale_val(g, power_scale),
                            Self::scale_val(b, power_scale),
                        );

                        // Leds are [B, G, R, W] ordering
                        leds[spine_hard * LEDS_PER_SPINE + led] = [b, g, r, 0];
                    }
                }
                let leds = controller.leds_mut(1);
                // spine_hard represents a physical LED connector on the PCB
                for spine_hard in (SPINES / 2)..SPINES {
                    // Figure out which logical spine this connector maps to
                    let spine_logical = map[spine_hard] - 1;
                    for led in 0..LEDS_PER_SPINE {
                        let (r, g, b) = (
                            led_update.spines[spine_logical][led][0],
                            led_update.spines[spine_logical][led][1],
                            led_update.spines[spine_logical][led][2],
                        );

                        // Apply power scaling if necessary
                        let (r, g, b) = (
                            Self::scale_val(r, power_scale),
                            Self::scale_val(g, power_scale),
                            Self::scale_val(b, power_scale),
                        );

                        // Leds are [B, G, R, W] ordering
                        leds[(spine_hard - (SPINES / 2)) * LEDS_PER_SPINE + led] = [b, g, r, 0];
                    }
                }
                controller.render()?;
            }
        }
    }

    /// Perform a quick test of the peripheral.  Must be called before start_thread.
    #[allow(dead_code)]
    pub fn test(&self) -> Result<()> {
        if self.thread_started {
            return Err(anyhow!(
                "Cannot perform test after peripheral thread is running."
            ));
        }

        println!("Testing WS2812b LED controller");
        let mut controller = get_controller()?;

        Self::set_all_leds(&mut controller, [0, 0, 255, 0]); // red
        thread::sleep(time::Duration::from_millis(300));
        Self::set_all_leds(&mut controller, [0, 255, 0, 0]); // green
        thread::sleep(time::Duration::from_millis(300));
        Self::set_all_leds(&mut controller, [255, 0, 0, 0]); // blue

        println!("Finished testing WS2812b LED controller");

        Ok(())
    }

    /// Set all LEDs in this controller to the same colour
    fn set_all_leds(controller: &mut Controller, argb: [u8; 4]) {
        let leds0 = controller.leds_mut(0);
        for led in leds0 {
            *led = argb;
        }
        let leds1 = controller.leds_mut(1);
        for led in leds1 {
            *led = argb;
        }
        controller.render().unwrap();
    }

    /// Function to be called in the main thread - sends a new LED state to
    /// the LED thread where it is passed to the hardware
    pub fn led_update(&self, leds: &LedUpdate) -> Result<()> {
        self.tx.send(leds.clone())?;
        Ok(())
    }

    /// Get the mapping from physical PCB connectors to spine positions. The
    /// mapping is loaded from the config file.  Each position in the array
    /// corresponds to a PCB connector (numebered 1-12 inclusive) and each
    /// value in the array is the spine position (numbered 1-12 inclusive).
    /// Spine positions are defined by the web visualiser (and also appear in
    /// geometry.rs)
    fn get_led_mapping() -> Result<[usize; 12]> {
        let map: [usize; 12] = SETTINGS.get("led_spine_mapping")?;

        // Check the mapping is valid
        let mut map_sorted = map;
        map_sorted.sort_unstable();
        assert_eq!(map_sorted, [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12]);

        Ok(map)
    }

    /// Work out how much current will be consumed by the LEDs in the
    /// requested illuminations.  If this exceeds or nears the maximum
    /// allowable value then work out a scaling to bring them back in range.
    /// If no scaling is required then None is returned.
    fn get_power_limit_scaling(leds: &LedUpdate) -> Option<f32> {
        // We work out the current as follows:
        // - There is a constant offset current for powering the WS2812b
        //   on-chip controllers (assume no auto-off function)
        // - Assume the LEDs are linear, i.e. that a subpixel value of 255
        //   consumes 255 times more current than a value of 1 (this is backed
        //   up by experiment)
        // - Use different current-per-value scalings for each of R, G, and B
        //   (they are more similar than I expected but slightly different)

        // Actual measurements, all at 15V rail with LED brightness 128/255
        // With all LEDs fully off, so just the Pi: -0.2A
        // With one LED very slightly on, so no auto-off: -0.5A
        // With one spine (118 LEDs) at full red: -0.75A
        // With one spine (118 LEDs) at full green: -0.75A
        // With one spine (118 LEDs) at full blue: -0.75A
        // 4x spines full red: -1.61A
        // 4x spines full green: -1.5A
        // 4x spines full blue: -1.62A
        // 1x spine at full white: -1.3A

        // Values determined by measurement, all in amps.  All are per logical
        // pixel, i.e. per two physical (doubled up) LED pixels.
        let current_offset: f32 = 1.5; // Approximate guess
        let current_per_val_r: f32 = 0.00005533; // Approximate guess
        let current_per_val_g: f32 = 0.00004985; // Approximate guess
        let current_per_val_b: f32 = 0.00005533; // Approximate guess

        let mut total_current: f32 = current_offset;
        for spine in leds.spines.iter() {
            for led in spine.iter() {
                total_current += led[0] as f32 * current_per_val_r;
                total_current += led[1] as f32 * current_per_val_g;
                total_current += led[2] as f32 * current_per_val_b;
            }
        }

        // Our 5V DC-DC converter is limited to 5A output, let's limit to 4A
        // for safety. There are various nice smooth limiting curves we could
        // use, but for now just do a hard compressor: If the current would
        // exceed 5A then apply scaling to all LEDs such that it equals 5A.
        let limit: f32 = 4.0;
        if total_current <= limit {
            None
        } else {
            Some(limit / total_current)
        }
    }

    /// Apply scaling to a subpixel value, if required
    fn scale_val(value: u8, scaling: Option<f32>) -> u8 {
        if let Some(scaling) = scaling {
            f32::round(value as f32 * scaling) as u8
        } else {
            value
        }
    }
}
