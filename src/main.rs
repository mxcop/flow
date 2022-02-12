use std::{env, io::Error, io::Write, net::SocketAddr, sync::Arc};

#[macro_use]
extern crate log;

use futures_util::{StreamExt, stream::SplitSink};
use send::send_all;
use serde_json::{Value, json};
use tokio::{net::{TcpListener, TcpStream}, sync::Mutex};
use tokio_tungstenite::{WebSocketStream, tungstenite::Message};

use colored::*;
mod info;
mod trafic;
mod send;
mod util;

pub static mut USERS: Vec<FluxUser> = Vec::new();

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
    let addr = stream
        .peer_addr()
        .expect("connected streams should have a peer address");
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
                        Err(_) => info::user_info(
                            addr,
                            String::from("Invalid (Needs to be JSON)"),
                            Color::Red
                        ),
                    };
                }
            },
            Err(_) => {
                // Handle user disconnect:
                remove_user(addr).await;
                panic!("");
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
            match msg_type.as_str() {
                "login" => trafic::login(json, addr, socket).await,
                "chat"  => trafic::chat(json, addr).await,

                _ => info::user_info(
                    addr,
                    format!("Invalid (Unknown type \"{}\")", msg_type),
                    Color::Red
                ),
            }
        }

        _ => info::user_info(
            addr,
            String::from("Invalid (Missing type)"),
            Color::Red
        ),
    }

}