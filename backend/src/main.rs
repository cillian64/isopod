use warp::Filter;
use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
struct Fix {
    long: f64,
    lat: f64,
    alt: f32,
    time: String,
    sats: usize,
}

#[tokio::main]
async fn main() {
    let isopod = warp::post()
        .and(warp::path("isopod"))
        // Only accept bodies smaller than 16kb
        .and(warp::body::content_length_limit(1024 * 16))
        .and(warp::body::json())
        .map(|fix: Fix| {
            println!("Rx: {:?}", fix);
            warp::reply()
        });

    warp::serve(isopod).run(([0, 0, 0, 0], 1309)).await

}

