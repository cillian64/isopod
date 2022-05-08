//! Reports location and other information to the backend server

use anyhow::Result;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;
use ureq::Agent;

use crate::common_structs::BatteryReadings;
use crate::common_structs::GpsFix;
use crate::temperature::get_temperature;

pub struct Reporter {
    tx: mpsc::Sender<(Option<GpsFix>, BatteryReadings)>,
}

impl Reporter {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel();
        thread::Builder::new()
            .name("ISOPOD reporter".into())
            .spawn(move || Self::reporter_thread(rx))
            .unwrap();
        Self { tx }
    }

    fn reporter_thread(rx: mpsc::Receiver<(Option<GpsFix>, BatteryReadings)>) -> ! {
        let agent: Agent = ureq::AgentBuilder::new()
            .timeout_read(Duration::from_secs(5))
            .timeout_write(Duration::from_secs(5))
            .build();

        println!("Reporter thread started up.");

        loop {
            let (fix, battery_readings) = rx.recv().unwrap();
            let fix = fix.unwrap_or_else(|| {
                println!("Stubbing out fix because no GPS signal yet");
                GpsFix::default()
            });

            let temperature = get_temperature()
                .map(|degrees| format!("{}°C", degrees))
                .unwrap_or_else(|| "unknown".to_owned());

            // Ignore errors here, just try again next time.
            let datetime = fix.time.format("%Y-%m-%d %H:%M:%S").to_string();
            let _ = agent
                .post("http://dwt27.co.uk:1309/isopod")
                .send_json(ureq::json!({
                    "lat": fix.latitude,
                    "long": fix.longitude,
                    "sats": fix.satellites,
                    "alt": fix.altitude,
                    "time": datetime,
                    "voltage": battery_readings.voltage,
                    "current": battery_readings.current,
                    "soc": battery_readings.soc,
                    "temp": temperature,
                }));

            println!(
                "Reporter thread sending fix: {:#?} battery {:#?} temperature {}",
                fix, battery_readings, temperature
            );
        }
    }

    #[allow(dead_code)]
    pub fn send(&mut self, fix: Option<GpsFix>, battery: BatteryReadings) -> Result<()> {
        self.tx.send((fix, battery))?;
        Ok(())
    }
}
