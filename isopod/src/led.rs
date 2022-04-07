//! Controls the attached addressable LEDs using the PWM and GPIO peripherals.

use anyhow::{anyhow, Result};
use rppal::gpio::Gpio;
use std::sync::{Arc, Mutex};

struct LedInternal {
    _gpio: Gpio,
    thread_started: bool,
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
            }),
        }
    }

    /// Start up a new thread controlling this peripheral.
    pub fn start_thread(self: Arc<Self>) -> () {
        let thread_started = self.internal.lock().unwrap().thread_started;
        if !thread_started {
            std::thread::spawn(move || self.led_thread());
        }
    }

    /// The main peripheral control thread
    fn led_thread(self: &Self) -> ! {
        {
            let mut internal = self.internal.lock().unwrap();
            internal.thread_started = true;
        }
        loop {
            unimplemented!();
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
        unimplemented!();
    }

    pub fn set(self: &Self) -> () {
        unimplemented!();
    }
}
