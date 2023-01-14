use axum::{
    extract::State,
    routing::{get, post},
    Json, Router,
};
use local_ip_address::local_ip;
use serde_derive::Deserialize;
use std::{fs, net::SocketAddr};
use std::{process, time::Duration};
use tokio::time::sleep;

const CONFIG_DATA: &str = r#""port" = 3000
"auth" = ""

"id" = 0
"token" = ""
"#;

#[tokio::main]
async fn main() {
    //get config
    let config_file = match fs::read_to_string("config.toml") {
        Ok(file) => file,
        Err(_) => {
            fs::write("./config.toml", CONFIG_DATA)
                .expect("НЕ УДАЛОСЬ ЗАПИСАТЬ СОЗДАТЬ config.toml, ПРОВЕРЬТЕ ПРАВА");
            println!("Заполни config файл и перезапусти службу\n");

            process::exit(exitcode::OK);
        }
    };

    let config: Config = toml::from_str(&config_file).expect("НЕПРАВИЛЬНО ЗАПОЛНЕН config.toml");
    if config.auth.is_empty() || config.token.is_empty() || config.id == 0 || config.port == 0 {
        println!("Все поля config файла должны быть заполнены\n");

        process::exit(exitcode::OK);
    }

    //get ip
    let addr = format!("{}:{}", local_ip().unwrap(), config.port);
    let socket_address: SocketAddr = addr.parse().expect("НЕПРАВИЛЬНЫЙ АДРЕС");

    //paths
    let app = Router::new()
        .route("/message", post(send_message))
        .with_state(config)
        .route("/", get(hello))
        .route("/exit", get(exit));

    //run server
    println!("Running on {}", &socket_address);
    axum::Server::bind(&socket_address)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

#[derive(Deserialize, Clone, Debug)]
struct Config {
    port: u32,
    auth: String,
    id: i32,
    token: String,
}

#[derive(Deserialize)]
struct CreateMessage {
    auth: String,
    message: String,
}

async fn hello() -> &'static str {
    sleep(Duration::from_millis(300)).await;
    return "Hello World\n";
}

async fn send_message(
    State(config): State<Config>,
    Json(payload): Json<CreateMessage>,
) -> &'static str {
    sleep(Duration::from_millis(500)).await;
    if payload.auth != config.auth {
        return "Неправильный код авторизации\n";
    }

    println!("{}", payload.message);
    return "message\n";
}

async fn exit() {
    process::exit(exitcode::OK);
}
