use std::sync::RwLock;
use warp::Filter;
use std::collections::HashMap;
use lazy_static::lazy_static;

pub struct Controls {
    // Soft brightness as a proportion of the value in settings.toml.  Expected
    // to be 0-100 inclusive, taken as a %
    pub brightness: u8,
}

impl Default for Controls {
    fn default() -> Self {
        Self {
            brightness: 0,
        }
    }
}

lazy_static! {
    pub static ref CONTROLS: RwLock<Controls> = RwLock::new(Controls::default());
}


async fn control_server() {
    let index = warp::get()
        .and(warp::path::end())
        .and(warp::fs::file("./index.html"));
    let index2 = warp::get()
        .and(warp::path("index.html"))
        .and(warp::fs::file("./index.html"));

    let bootstrap = warp::get()
        .and(warp::path("bootstrap.css"))
        .and(warp::path::end())
        .and(warp::fs::file("./bootstrap.css"));

    let command = warp::any()
        .and(warp::path("command"))
        .and(warp::body::content_length_limit(4096))
        .and(warp::body::form())
        .map(|p: HashMap<String, String>| {
            eprintln!("Got query strings: {:?}", p);

            if let Some(x) = p.get("brightness") {
                if let Ok(x) = x.parse::<u8>() {
                    if x <= 100 {
                        CONTROLS.write().unwrap().brightness = x;
                    }
                }
            }

            warp::reply()
        });

    let routes = index
        .or(index2)
        .or(bootstrap)
        .or(command);

    warp::serve(routes).run(([0, 0, 0, 0], 80)).await;
}

pub fn start_server() {
    std::thread::spawn(move || {
        println!("Starting control server...");
        tokio::runtime::Runtime::new().unwrap().block_on(control_server());
    });
}
