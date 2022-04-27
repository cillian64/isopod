//! Handles peripherals connected over I2C (the IMU and the battery fuel
//! gauge).

use crate::common_structs::{BatteryReadings, ImuReadings};
use anyhow::{anyhow, Result};
use linux_embedded_hal as hal;
use max1720x::MAX1720x;
use rppal::i2c::I2c;
use std::ops::DerefMut;
use std::sync::{Arc, Mutex};
use std::{thread, time};

struct I2cPeriphsInternal {
    thread_started: bool,
    imu: ImuReadings,
    battery: BatteryReadings,
}

pub struct I2cPeriphs {
    // Held in a separate mutex because the i2c thread will need to hold it
    // constantly
    i2c: Mutex<I2c>,

    // Mainly used to get readings from the sensor reading thread to the main
    // thread.
    internal: Mutex<I2cPeriphsInternal>,
}

impl I2cPeriphs {
    pub fn new(i2c: I2c) -> Self {
        Self {
            i2c: Mutex::new(i2c),
            internal: Mutex::new(I2cPeriphsInternal {
                thread_started: false,
                imu: ImuReadings::default(),
                battery: BatteryReadings::default(),
            }),
        }
    }

    #[allow(dead_code)]
    pub fn test(&self) -> Result<()> {
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
            std::thread::Builder::new()
                .name("ISOPOD I2C".into())
                .spawn(move || self.i2c_thread())
                .unwrap();
        }
    }

    fn i2c_thread(&self) -> ! {
        {
            let mut internal = self.internal.lock().unwrap();
            internal.thread_started = true;
        }
        println!("I2C thread running.");

        let mut i2c = self.i2c.lock().unwrap();
        let mut icm = icm20948::ICMI2C::<_, _, 0x69>::new(i2c.deref_mut()).unwrap();
        icm.init(i2c.deref_mut(), &mut hal::Delay).unwrap();
        let mut max17205 = MAX1720x::new(i2c.deref_mut());

        loop {
            // Try to get some new readings from the IMU.  If we succeed, then
            // convert them to SI units and store them.  If we fail to get new
            // readings then just try again next time.  In prototyping this can
            // happen if the bodge-wires lose contact.
            if let Ok(raw) = icm.get_values_accel_gyro(i2c.deref_mut()) {
                let (xa, ya, za, xg, yg, zg) = icm.scale_raw_accel_gyro(raw);
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
            };

            // Fetch the current readings from the fuel gauge. Ignore any errors.
            if let Ok(soc) = max17205.state_of_charge(i2c.deref_mut()) {
                self.internal.lock().unwrap().battery.soc = soc;
            }
            if let Ok(voltage) = max17205.voltage(i2c.deref_mut()) {
                self.internal.lock().unwrap().battery.voltage = voltage;
            }
            if let Ok(current) = max17205.current(i2c.deref_mut()) {
                self.internal.lock().unwrap().battery.current = current;
            }

            thread::sleep(time::Duration::from_millis(1000));
        }
    }

    pub fn get_imu(&self) -> ImuReadings {
        self.internal.lock().unwrap().imu
    }

    pub fn get_battery(&self) -> BatteryReadings {
        self.internal.lock().unwrap().battery
    }
}
