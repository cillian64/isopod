use crate::common_structs::LedUpdate;
use anyhow::Result;
use futures_util::SinkExt;
use serde::Serialize;
use std::sync::{mpsc, Arc, Mutex};
use tokio::sync::broadcast;
use tokio::sync::broadcast::{Receiver, Sender};
use warp::{ws::WebSocket, Filter};

/// The packet format we send to the websocket client
#[derive(Serialize, Clone)]
struct SimPacket {
    spines: Vec<Vec<[u8; 3]>>,
}

pub struct WsServer {
    // This channel goes from the main thread to the JSONifier
    tx: mpsc::Sender<LedUpdate>,
}

impl WsServer {
    pub fn start_server() -> Self {
        let (tx, _rx) = broadcast::channel(32);

        // Wrap up a tokio Sender so it can live forever
        let wrapped_tx: Arc<Mutex<Sender<String>>> = Arc::new(Mutex::new(tx));
        let wrapped_tx2 = wrapped_tx.clone();
        // Make a new warp filter which provides our state - a tokio sender from
        // which we can spawn more receivers.
        let wrapped_tx_filter = warp::any().map(move || wrapped_tx2.clone());

        std::thread::spawn(move || {
            let routes = warp::path("ws").and(warp::ws()).and(wrapped_tx_filter).map(
                |ws: warp::ws::Ws, tx: Arc<Mutex<Sender<String>>>| {
                    ws.on_upgrade(move |socket| {
                        user_connected(socket, tx.lock().unwrap().subscribe())
                    })
                },
            );

            println!("Starting websocket listener...");
            let future = async move {
                warp::serve(routes).run(([0, 0, 0, 0], 3030)).await;
            };
            tokio::runtime::Runtime::new().unwrap().block_on(future);
        });

        // Spawn the JSONifier thread which receives LedUpdates and converts
        // then to JSON messages.  This is CPU intensive so we don't want it
        // duplicated in every websocket handler but also don't want to burden
        // the main thread with it, so it's done by a dedicated thread.
        let (jsonifier_tx, jsonifier_rx) = mpsc::channel::<LedUpdate>();
        std::thread::Builder::new()
            .name("ISOPOD JSONifier".into())
            .spawn(move || {
                loop {
                    let leds = jsonifier_rx.recv().unwrap();
                    let packet = SimPacket {
                        spines: leds.spines.clone(),
                    };
                    let packet_json = serde_json::to_string(&packet).unwrap();

                    // We mysteriously get channel closed errors occasionally.
                    // Not sure why, so just ignore any errors sending into the
                    // channel.
                    let _res = wrapped_tx.lock().unwrap().send(packet_json);
                }
            })
            .unwrap();

        Self { tx: jsonifier_tx }
    }

    pub fn led_update(&self, leds: &LedUpdate) -> Result<()> {
        // TODO: JSONifying the LED state at 60fps takes about 60% of a
        // raspberry pi 3 core.  Is there a more computationally efficient
        // way to send this data to the visualiser frontend?

        self.tx.send(leds.clone())?;

        Ok(())
    }
}

async fn user_connected(mut ws: WebSocket, mut rx: Receiver<String>) {
    println!("Websocket connected.");

    loop {
        // Wait for an LED state update.
        let packet_json = rx.recv().await.unwrap();

        let message = warp::ws::Message::text(&packet_json);

        // Send the WS packet to the client
        match ws.send(message).await {
            Ok(_) => {}
            Err(_) => {
                println!("Websocket disconnected.");
                break;
            }
        };
    }
}
