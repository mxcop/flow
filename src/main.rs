use std::{env, io::Error, io::Write, net::SocketAddr, sync::Arc};

#[macro_use]
extern crate log;

use futures_util::{StreamExt, stream::SplitSink};
use send::send_all;
use serde_json::{Value, json};
use tokio::{net::{TcpListener, TcpStream}, sync::Mutex};
use tokio_tungstenite::{WebSocketStream, tungstenite::Message};

use colored::*;
use utils::fuppercase;
mod info;
mod trafic;
mod send;
mod utils;

pub static mut USERS: Vec<FluxUser> = Vec::new();
pub static mut OFFERS: Vec<Offer> = Vec::new();

#[derive(Debug)]
pub struct Offer {
    origin: String,
    target: String,
    id: String
}

#[derive(Debug)]
pub struct FluxUser {
    id: String,
    name: String,
    addr: SocketAddr,
    socket: Arc<Mutex<SplitSink<WebSocketStream<TcpStream>, Message>>>
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Initialise the logging environment.
    env_logger::builder()
        .format(|buf, record| writeln!(buf, "{}", record.args()))
        .filter_level(log::LevelFilter::Info)
        .init();

    // Get the address from the args.
    let addr = env::args()
        .nth(1)
        .unwrap_or_else(|| "192.168.178.53:25656".to_string());

    // Start the server by creating the TcpListener.
    info::info("Startup".white(), String::from("Starting the server..."));
    let try_socket = TcpListener::bind(&addr).await;
    let listener = try_socket.expect("Failed to bind to address");
    info::info("Started".white(), format!("Listening on: {}", addr));

    // Keep waiting for new connections.
    while let Ok((stream, _)) = listener.accept().await {
        // When a connection is made spawn a new thread for it.
        tokio::spawn(accept_connection(stream));
    }

    Ok(())
}

/**
 * Called when a new connection is made to the server.
 */
async fn accept_connection(stream: TcpStream) {
    let addr = stream.peer_addr().expect("Failed to get user public ip");

    info::info("Connection".blue(), addr.to_string());

    // Perform the websocket handshake.
    let ws_stream = tokio_tungstenite::accept_async(stream)
        .await
        .expect("Error during the websocket handshake occurred");

    info::info("Handshaked".green(), addr.to_string());

    // Split the streams write and read.
    let (write, read) = ws_stream.split();
    let writer = Arc::new(Mutex::new(write));

    // Read incoming messages and process them:
    read.for_each(|message| async {
        match message {
            Ok(msg) => {
                let writer_cl = Arc::clone(&writer);

                // Check if the message isn't empty.
                if msg.len() > 0 {
                    let data = serde_json::from_str::<Value>(msg.to_string().as_str());

                    // Check if the message is valid JSON:
                    match data {
                        Ok(json) => validate_json(json, addr, writer_cl).await,
                        Err(_) => info::user_err(
                            addr,
                            String::from("Json -> Invalid message format")
                        ),
                    };
                }
            },
            Err(err) => {
                // Handle user disconnect:
                remove_user(addr).await;
                panic!("{}", err);
            }
        }
    }).await;

    // Handle user disconnect:
    remove_user(addr).await;
}

/**
 * Remove a user by their socket address.
 */
async fn remove_user(addr: SocketAddr) {
    unsafe {
        let index = USERS.iter().position(|user| user.addr == addr);

        match index {
            Some(i) => {
                info::info("Disconnected".red(), String::clone(&USERS.get(i).expect("Can get user when disconnected").name));

                // Send an update to all other users that a user has left:
                let user = USERS.get(i).expect("Can read user");
                let update_json = json!({
                    "type": "leave",
                    "user": {
                        "id": user.id,
                        "name": user.name
                    }
                });

                send_all(addr, update_json.to_string()).await;

                // Cancel any connected offers:
                OFFERS.retain(|offer| offer.origin != user.id && offer.target != user.id);

                USERS.remove(i);
                ()
            },
            None => info::info("Hard Disconnect".red(), addr.to_string()),
        }
    }
}

/**
 * Validates the json message and its contents.
 */
async fn validate_json(json: Value, addr: SocketAddr, socket: Arc<Mutex<SplitSink<WebSocketStream<TcpStream>, Message>>>) {

    // Check if type exists on the message:
    match &json["type"] {
        Value::String(msg_type) => {

            // Check which type this message is:
            let err = match msg_type.as_str() {
                "login" => trafic::login(json.clone(), addr, socket).await,
                "chat"  => trafic::chat(json.clone(), addr).await,
                "file"  => trafic::file(json.clone(), addr).await,
                "request" => trafic::request(json.clone(), addr).await,     // Request for p2p
                "offer" => trafic::offer(json.clone(), addr).await,         // P2P offer
                "session" => trafic::session(json.clone(), addr).await,     // P2P session info

                _ => Some(format!("Invalid (Unknown type \"{}\")", msg_type))
            };

            // Log the error if there is one:
            match err {
                Some(err) => info::user_err(
                    addr,
                    format!("{} -> {}", fuppercase(msg_type), err)
                ),
                None => {}
            }
        }

        _ => info::user_err(
            addr,
            String::from("Json -> Missing type field")
        ),
    }

}