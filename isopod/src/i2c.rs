//! Handles peripherals connected over I2C (the IMU and the battery fuel
//! gauge).

use anyhow::{anyhow, Result};
use linux_embedded_hal as hal;
use rppal::i2c::I2c;
use std::sync::{Arc, Mutex};
use std::{thread, time};

struct I2cPeriphsInternal {
    i2c: I2c,
    thread_started: bool,
}

pub struct I2cPeriphs {
    internal: Mutex<I2cPeriphsInternal>,
}

impl I2cPeriphs {
    pub fn new(i2c: I2c) -> Self {
        Self {
            internal: Mutex::new(I2cPeriphsInternal {
                i2c,
                thread_started: false,
            }),
        }
    }

    pub fn test(self: &Self) -> Result<()> {
        let mut internal = self.internal.lock().unwrap();
        if internal.thread_started {
            return Err(anyhow!(
                "Cannot perform test after peripheral thread is running."
            ));
        }

        println!("Testing I2C and IMU...");
        let mut icm = icm20948::ICMI2C::<_, _, 0x69>::new(&mut internal.i2c)?;
        icm.init(&mut internal.i2c, &mut hal::Delay).unwrap();
        for _ in 0..3 {
            let (xa, ya, za, xg, yg, zg) =
                icm.scale_raw_accel_gyro(icm.get_values_accel_gyro(&mut internal.i2c).unwrap());
            println!(
                "Sensed, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}",
                xa, ya, za, xg, yg, zg
            );
            thread::sleep(time::Duration::from_secs(1));
        }
        println!("I2C and IMU ok!");
        Ok(())
    }

    pub fn start_thread(self: Arc<Self>) {
        let thread_started = self.internal.lock().unwrap().thread_started;
        if !thread_started {
            std::thread::spawn(move || self.i2c_thread());
        }
    }

    fn i2c_thread(self: &Self) -> ! {
        {
            let mut internal = self.internal.lock().unwrap();
            internal.thread_started = true;
        }
        loop {
            unimplemented!();
        }
    }

    pub fn get(self: &Self) {
        unimplemented!();
    }
}
