use warp::{Filter, ws::WebSocket};
use tokio::time::{sleep, Duration};
use futures_util::SinkExt;
use serde::Serialize;

#[derive(Serialize)]
struct SimPacket {
    spines: Vec<Vec<[u8; 3]>>,
}

#[tokio::main]
async fn main() {
    let routes = warp::path("ws")
        // The `ws()` filter will prepare the Websocket handshake.
        .and(warp::ws())
        .map(|ws: warp::ws::Ws| {
            println!("Got connection to WS route, trying to upgrade...");
            // And then our closure will be called when it completes...
            ws.on_upgrade(move |socket| user_connected(socket))
        });

    println!("Setting up listener");
    warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
    println!("Listener should be setup.");
}

async fn user_connected(mut ws: WebSocket) {
    println!("New user connected.");

    let mut i = 59;
    loop {
        // Decide on LED colours
        let mut led_colours = vec![vec![[0, 0, 0]; 60]; 12];
        for spine in 0..12 {
            for led in 0..60 {
                led_colours[spine][led] = if (led + i) % 10 == 0 {
                    [255, 255, 255]
                } else {
                    [0, 0, 0]
                };
            }
        }
        i = if i == 0 { 60 } else { i - 1 };

        // Build packet
        let packet = SimPacket { spines: led_colours };
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

        // Sleep until the next frame
        let sleep = sleep(Duration::from_millis(1000 / 60));
        sleep.await;
    }
}
