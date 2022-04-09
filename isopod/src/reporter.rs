//! Reports location and other information to the backend server

use std::thread;
use std::sync::mpsc;
use anyhow::Result;
use ureq::Agent;
use std::time::Duration;

use crate::gps::Fix;

pub struct Reporter {
    tx: mpsc::Sender<Option<Fix>>,
}

impl Reporter {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::channel();
        thread::spawn(move || Self::reporter_thread(rx));
        Self {
            tx,
        }
    }

    fn reporter_thread(rx: mpsc::Receiver<Option<Fix>>) -> ! {
        let agent: Agent = ureq::AgentBuilder::new()
            .timeout_read(Duration::from_secs(5))
            .timeout_write(Duration::from_secs(5))
            .build();

        println!("Reporter thread started up.");

        loop {
            let fix = match rx.recv().unwrap() {
                Some(fix) => fix,
                // For now don't send updates with no fix.
                None => continue,
            };

            // Ignore errors here, just try again next time.
            let datetime = fix.time.format("%Y-%m-%d %H:%M:%S").to_string();
            let _ = agent
                .put("https://dwt27.co.uk/isopod")
                .send_json(ureq::json!({
                    "lat": fix.latitude,
                    "long": fix.longitude,
                    "sats": fix.satellites,
                    "alt": fix.altitude,
                    "time": datetime,
                }));

            println!("Reporter thread sending fix: {:?}", fix);
        }
    }

    pub fn send(self: &mut Self, fix: Option<Fix>) -> Result<()> {
        self.tx.send(fix)?;
        Ok(())
    }
}