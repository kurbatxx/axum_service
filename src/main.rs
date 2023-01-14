use axum::{
    routing::{get, post},
    Json, Router,
};
use local_ip_address::local_ip;
use serde_derive::Deserialize;
use std::time::Duration;
use std::{fs, net::SocketAddr};
use tokio::time::sleep;

#[tokio::main]
async fn main() {
    //get ip
    let addr = format!("{}:{}", local_ip().unwrap(), 3000);
    let socket_address: SocketAddr = addr.parse().expect("НЕПРАВИЛЬНЫЙ АДРЕС");

    //paths
    let app = Router::new()
        .route("/", get(hello))
        .route("/message", post(send_message));

    //run server
    println!("Running on {}", &socket_address);
    axum::Server::bind(&socket_address)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

#[derive(Deserialize)]
struct CreateMessage {
    message: String,
}

async fn hello() -> &'static str {
    sleep(Duration::from_millis(300)).await;
    return "Hello World\n";
}

async fn send_message(Json(payload): Json<CreateMessage>) -> &'static str {
    sleep(Duration::from_millis(500)).await;
    println!("{}", payload.message);
    return "message\n";
}
