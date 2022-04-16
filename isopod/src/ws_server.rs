use warp::{Filter, ws::WebSocket};
use futures_util::SinkExt;
use serde::Serialize;
use tokio::sync::broadcast;
use tokio::sync::broadcast::{Sender, Receiver};
use crate::led::LedUpdate;
use std::sync::{Arc, Mutex};
use anyhow::Result;

/// The packet format we send to the websocket client
#[derive(Serialize, Clone)]
struct SimPacket {
    spines: Vec<Vec<[u8; 3]>>,
}

pub struct WsServer {
    tx: Arc<Mutex<Sender<LedUpdate>>>,
}

impl WsServer {
    pub fn start_server() -> Self {
        let (tx, _rx) = broadcast::channel(32);

        // Wrap up a tokio Sender so it can live forever
        let wrapped_tx: Arc<Mutex<Sender<LedUpdate>>> = Arc::new(Mutex::new(tx));
        let wrapped_tx2 = wrapped_tx.clone();
        // Make a new warp filter which provides our state - a tokio sender from
        // which we can spawn more receivers.
        let wrapped_tx_filter = warp::any().map(move || wrapped_tx2.clone());

        std::thread::spawn(move || {
            let routes = warp::path("ws")
                .and(warp::ws())
                .and(wrapped_tx_filter)
                .map(|ws: warp::ws::Ws, tx: Arc<Mutex<Sender<LedUpdate>>>| {
                     ws.on_upgrade(move |socket| {
                         user_connected(socket, tx.lock().unwrap().subscribe())
                     })
                });

            println!("Starting websocket listener...");
            let future = async move {
                warp::serve(routes).run(([0, 0, 0, 0], 3030)).await;
            };
            tokio::runtime::Runtime::new()
                .unwrap()
                .block_on(future);
        });
        Self { tx: wrapped_tx }
    }

    pub fn led_update(self: &Self, leds: &LedUpdate) -> Result<()> {
        // We mysteriously get channel closed errors occasionally.  Not sure
        // why, so just ignore any errors sending into the channel.
        let _res = self.tx.lock().unwrap().send(leds.clone());
        Ok(())
    }
}


async fn user_connected(mut ws: WebSocket, mut rx: Receiver<LedUpdate>) {
    println!("Websocket connected.");

    loop {
        // Wait for an LED state update.
        let led_update = rx.recv().await.unwrap();

        // Build packet
        let packet = SimPacket { spines: led_update.spines };
        // TODO: JSONifying the LED state at 60fps takes about 60% of a
        // raspberry pi 3 core.  Is there a more computationally efficient
        // way to send this data to the visualiser frontend?
        let packet_json = serde_json::to_string(&packet).unwrap();
        let message = warp::ws::Message::text(&packet_json);

        // Send the WS packet to the client
        match ws.send(message).await {
            Ok(_) => {},
            Err(_) => {
                println!("Websocket disconnected.");
                break;
            }
        };
    }
}
