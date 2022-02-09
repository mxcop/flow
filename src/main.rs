use std::{env, io::Error};

#[macro_use]
extern crate log;
use env_logger::Env;

use futures_util::{future, StreamExt, TryStreamExt};
use tokio::net::{TcpListener, TcpStream};

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Initialise the logging environment.
    env_logger::init_from_env(Env::default().filter_or("MY_LOG_LEVEL", "info").write_style_or("MY_LOG_STYLE", "always"));

    // Get the address from the args.
    let addr = env::args().nth(1).unwrap_or_else(|| "192.168.178.53:25656".to_string());

    // Start the server by creating the TcpListener.
    info!("Starting the server...");
    let try_socket = TcpListener::bind(&addr).await;
    let listener = try_socket.expect("Failed to bind to address");
    info!("Listening on: {}", addr);

    // Keep waiting for new connections.
    while let Ok((stream, _)) = listener.accept().await {
        // When a connection is made spawn a new thread for it.
        tokio::spawn(accept_connection(stream));
    }

    Ok(())
}

async fn accept_connection(stream: TcpStream) {
    let addr = stream.peer_addr().expect("connected streams should have a peer address");
    info!("[Connection] {}", addr);

    let ws_stream = tokio_tungstenite::accept_async(stream)
        .await
        .expect("Error during the websocket handshake occurred");

    info!("[Handshaked] {}", addr);

    let (write, read) = ws_stream.split();
    
    // We should not forward messages other than text or binary.
    read.try_filter(|msg| future::ready(msg.is_text() || msg.is_binary()))
        .forward(write)
        .await
        .expect("Failed to forward messages")

    // todo : find way to know that a connection was lost.
    // todo : do stuff with the incoming messages!
}