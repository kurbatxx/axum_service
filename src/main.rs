use axum::{
    extract::State,
    routing::{get, post},
    Json, Router,
};
use local_ip_address::local_ip;
use reqwest::Client;
use serde::{Deserialize, Serialize};
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
        // .set_time_offset_to_local()
        // .unwrap()
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
        //.use_native_tls()
        .use_rustls_tls()
        //.danger_accept_invalid_hostnames(true)
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
        .route("/info", post(info))
        .with_state(server_state)
        .route("/", get(check))
        .route("/exit", get(exit));

    //run server
    info!("Running on {}", &socket_address);
    axum::Server::try_bind(&socket_address)
        .unwrap_or_else(|err| {
            warn!(
                "Порт {} уже занят, завершите все процессы изпользующие данный порт",
                config.port
            );
            error!("{}", err);
            process::exit(exitcode::OK);
        })
        .serve(app.into_make_service())
        .await
        .unwrap_or_else(|err| {
            error!("{}", err);
            process::exit(exitcode::OK);
        });
}

#[derive(Deserialize, Clone, Debug)]
struct Config {
    port: u32,
    auth: String,
    id: i64,
    token: String,
}

#[derive(Deserialize)]
struct CreateMessage {
    auth: String,
    message: String,
}

#[derive(Deserialize, Serialize)]
struct MessageInfo {
    auth: String,
    message: Vec<Instance>,
}

#[derive(Clone, Debug)]
struct ServerState {
    client: Client,
    config: Config,
}

async fn check() -> &'static str {
    sleep(Duration::from_millis(1)).await;
    return "It works\n";
}

async fn send_message(
    State(state): State<ServerState>,
    Json(payload): Json<CreateMessage>,
) -> &'static str {
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

    dbg!(&req);

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

#[derive(Deserialize, Serialize, Debug)]
pub struct Instance {
    instance: String,
    backups: Vec<Backup>,
}

#[derive(Deserialize, Serialize, Debug)]
pub struct Backup {
    pub id: String,
    #[serde(rename(deserialize = "recovery-time"))]
    pub recovery_time: String,

    #[serde(rename(deserialize = "backup-mode"))]
    pub backup_mode: String,
    pub status: String,
}

async fn info(State(state): State<ServerState>, Json(payload): Json<MessageInfo>) -> &'static str {
    let config = state.config;
    if payload.auth != config.auth {
        return "Неправильный код авторизации\n";
    }

    let v_st_instances: Vec<String> = payload.message
                                .iter()
                                .map(|x| {
                                    let v_strings: Vec<String> = x
                                        .backups
                                        .iter()
                                        .map(|b| {
                                            format!(
                                                // %0A == \n
                                                "-------------------------------------------------------%0Aid: {}%0Arecovery time: {}%0Abackup mode: {}%0Astatus: {}%0A",
                                                b.id,
                                                b.recovery_time,
                                                b.backup_mode,
                                                b.status
                                            )
                                        })
                                        .collect();

                                    format!("Instance: <b>{}</b>%0A{}", x.instance, v_strings.join("\n"))
                                })
                                .collect();

    for text in v_st_instances {
        let req = state
            .client
            .get(
                format!(
                    "https://api.telegram.org/bot{}/sendMessage?chat_id={}&text={}&parse_mode=html",
                    config.token, config.id, text,
                )
                .as_str(),
            )
            .send()
            .await;

        match req {
            Ok(_) => {
                info!("{}", serde_json::to_string(&payload.message).unwrap());
                return "Отправлено!\n";
            }
            Err(err) => {
                warn!("{:?}", err);
                return "Не отправлено!\n";
            }
        }
    }
    return "Json Обрaботан\n";
}
