use axum::{
    extract::State,
    routing::{get, post},
    Json, Router,
};
use local_ip_address::local_ip;
use reqwest::Client;
use serde_derive::Deserialize;
use std::{
    fs::{self, OpenOptions},
    net::SocketAddr,
};
use std::{process, time::Duration};

use tokio::time::sleep;

#[macro_use]
extern crate log;
extern crate simplelog;
use simplelog::*;

const CONFIG_DATA: &str = r#""port" = 3000
"auth" = ""

"id" = 0
"token" = ""
"#;

#[tokio::main]
async fn main() {
    //init logger
    let logger_config = simplelog::ConfigBuilder::new()
        .set_time_format_custom(format_description!(
            "[day].[month].[year]  [hour]:[minute]:[second]"
        ))
        .set_time_offset_to_local()
        .unwrap()
        .build();

    CombinedLogger::init(vec![
        TermLogger::new(
            LevelFilter::Info,
            logger_config.clone(),
            TerminalMode::Mixed,
            ColorChoice::Auto,
        ),
        WriteLogger::new(
            LevelFilter::Info,
            logger_config,
            OpenOptions::new()
                .read(true)
                .write(true)
                .create(true)
                .append(true)
                .open("log_data.log")
                .unwrap(),
        ),
    ])
    .unwrap();

    //get config
    let config_file = match fs::read_to_string("config.toml") {
        Ok(file) => file,
        Err(_) => {
            fs::write("./config.toml", CONFIG_DATA)
                .expect("НЕ УДАЛОСЬ ЗАПИСАТЬ СОЗДАТЬ config.toml, ПРОВЕРЬТЕ ПРАВА");
            //println!("Заполни config файл и перезапусти службу\n");
            warn!("Не заполнен config, необходимо заполнить config.toml и перезапустить службу");
            process::exit(exitcode::OK);
        }
    };

    let config: Config = toml::from_str(&config_file).expect("НЕПРАВИЛЬНО ЗАПОЛНЕН config.toml");
    if config.auth.is_empty() || config.token.is_empty() || config.id == 0 || config.port == 0 {
        warn!("Не все поля config файла заполнены");
        process::exit(exitcode::OK);
    }

    //create client
    let client = reqwest::Client::builder()
        .use_native_tls()
        .danger_accept_invalid_hostnames(true)
        .danger_accept_invalid_certs(true)
        .build()
        .unwrap();

    let server_state = ServerState {
        client,
        config: config.clone(),
    };

    //get ip
    let addr = format!("{}:{}", local_ip().unwrap(), config.port);
    let socket_address: SocketAddr = addr.parse().expect("НЕПРАВИЛЬНЫЙ АДРЕС");

    //paths
    let app = Router::new()
        .route("/message", post(send_message))
        .with_state(server_state)
        .route("/", get(hello))
        .route("/exit", get(exit));

    //run server
    let server = axum::Server::bind(&socket_address).serve(app.into_make_service());

    if let Err(err) = server.await {
        error!("{}", err);
        process::exit(exitcode::OK);
    } else {
        info!("Running on {}\n", &socket_address)
    }
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

#[derive(Clone, Debug)]
struct ServerState {
    client: Client,
    config: Config,
}

async fn hello() -> &'static str {
    sleep(Duration::from_millis(1)).await;
    return "It works\n";
}

async fn send_message(
    State(state): State<ServerState>,
    Json(payload): Json<CreateMessage>,
) -> &'static str {
    //sleep(Duration::from_millis(500)).await;

    let config = state.config;
    if payload.auth != config.auth {
        return "Неправильный код авторизации\n";
    }

    let req = state
        .client
        .get(
            format!(
                "https://api.telegram.org/bot{}/sendMessage?chat_id={}&text={}",
                config.token,
                config.id,
                payload.message.to_string(),
            )
            .as_str(),
        )
        .send()
        .await;

    match req {
        Ok(_) => {
            info!("{}", payload.message);
            return "Отправлено!\n";
        }
        Err(err) => {
            warn!("{:?}", err);
            return "Не отправлено!\n";
        }
    }
}

async fn exit() {
    process::exit(exitcode::OK);
}
