//! Handles peripherals connected over I2C (the IMU and the battery fuel
//! gauge).

use anyhow::{anyhow, Result};
use linux_embedded_hal as hal;
use rppal::i2c::I2c;
use std::sync::{Arc, Mutex};
use std::{thread, time};
use std::ops::DerefMut;

struct I2cPeriphsInternal {
    thread_started: bool,
    imu: ImuReadings,
}

pub struct I2cPeriphs {
    // Held in a separate mutex because the i2c thread will need to hold it
    // constantly
    i2c: Mutex<I2c>,

    // Mainly used to get readings from the sensor reading thread to the main
    // thread.
    internal: Mutex<I2cPeriphsInternal>,
}

/// Represents the sensor data captured from the IMU at a given instant
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ImuReadings {
    /// Accelerometer X-axis reading in m/s/s
    pub xa: f32,
    /// Accelerometer Y-axis reading in m/s/s
    pub ya: f32,
    /// Accelerometer Z-axis reading in m/s/s
    pub za: f32,

    /// Gyroscope X-axis reading
    pub xg: f32,
    /// Gyroscope Y-axis reading
    pub yg: f32,
    /// Gyroscope Z-axis reading
    pub zg: f32,
}

impl I2cPeriphs {
    pub fn new(i2c: I2c) -> Self {
        Self {
            i2c: Mutex::new(i2c),
            internal: Mutex::new(I2cPeriphsInternal {
                thread_started: false,
                imu: ImuReadings {
                    xa: 0.0,
                    ya: 0.0,
                    za: 0.0,
                    xg: 0.0,
                    yg: 0.0,
                    zg: 0.0,
                },
            }),
        }
    }

    pub fn test(self: &Self) -> Result<()> {
        if self.internal.lock().unwrap().thread_started {
            return Err(anyhow!(
                "Cannot perform test after peripheral thread is running."
            ));
        }
        let mut i2c = self.i2c.lock().unwrap();

        println!("Testing I2C and IMU...");
        let mut icm = icm20948::ICMI2C::<_, _, 0x69>::new(i2c.deref_mut())?;
        icm.init(i2c.deref_mut(), &mut hal::Delay).unwrap();
        for _ in 0..3 {
            let (xa, ya, za, xg, yg, zg) =
                icm.scale_raw_accel_gyro(icm.get_values_accel_gyro(i2c.deref_mut()).unwrap());
            println!(
                "Sensed, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}",
                xa, ya, za, xg, yg, zg
            );
            thread::sleep(time::Duration::from_millis(300));
        }
        println!("I2C and IMU ok!");
        Ok(())
    }

    pub fn start_thread(self: Arc<Self>) {
        if !self.internal.lock().unwrap().thread_started {
            std::thread::spawn(move || self.i2c_thread());
        }
    }

    fn i2c_thread(self: &Self) -> ! {
        {
            let mut internal = self.internal.lock().unwrap();
            internal.thread_started = true;
        }
        println!("I2C thread running.");

        let mut i2c = self.i2c.lock().unwrap();
        let mut icm = icm20948::ICMI2C::<_, _, 0x69>::new(i2c.deref_mut()).unwrap();
        icm.init(i2c.deref_mut(), &mut hal::Delay).unwrap();

        loop {
            let (xa, ya, za, xg, yg, zg) =
                icm.scale_raw_accel_gyro(icm.get_values_accel_gyro(i2c.deref_mut()).unwrap());
            self.internal.lock().unwrap().imu = ImuReadings {
                xa,
                ya,
                za,
                xg,
                yg,
                zg,
            };
            // println!(
            //     "Sensed, {:?}, {:?}, {:?}, {:?}, {:?}, {:?}",
            //     xa, ya, za, xg, yg, zg
            // );
            thread::sleep(time::Duration::from_millis(100));
        }
    }

    pub fn get(&self) -> ImuReadings {
        self.internal.lock().unwrap().imu
    }
}
