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

pub fn start_server() -> Arc<Mutex<Sender<LedUpdate>>> {
    let (tx, _rx) = broadcast::channel(32);

    // Wrap up a tokio Sender so it can live forever
    let wrapped_tx: Arc<Mutex<Sender<LedUpdate>>> = Arc::new(Mutex::new(tx));
    let wrapped_tx2 = wrapped_tx.clone();
    // Make a new warp filter which provides our state - a tokio sender from
    // which we can spawn more receivers.
    let wrapped_tx_filter = warp::any().map(move || wrapped_tx2.clone());

    std::thread::spawn(move || {
        let routes = warp::path("ws")
            // The `ws()` filter will prepare the Websocket handshake.
            .and(warp::ws())
            .and(wrapped_tx_filter)
            .map(|ws: warp::ws::Ws, tx: Arc<Mutex<Sender<LedUpdate>>>| {
                println!("Got connection to WS route, trying to upgrade...");
                // And then our closure will be called when it completes...
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
    wrapped_tx
}

async fn user_connected(mut ws: WebSocket, mut rx: Receiver<LedUpdate>) {
    println!("New user connected.");

    loop {
        // Wait for an LED state update.
        let led_update = rx.recv().await.unwrap();

        // Build packet
        let packet = SimPacket { spines: led_update.spines };
        let packet_json = serde_json::to_string(&packet).unwrap();
        let message = warp::ws::Message::text(&packet_json);

        // Send the WS packet to the client
        match ws.send(message).await {
            Ok(_) => {},
            Err(_) => {
                println!("Client disconnected.");
                break;
            }
        };
    }
}

pub fn led_update(tx: Arc<Mutex<Sender<LedUpdate>>>, leds: &LedUpdate) -> Result<()> {
    tx.lock().unwrap().send(leds.clone())?;
    Ok(())
}
