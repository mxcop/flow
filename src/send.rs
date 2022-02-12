use std::net::SocketAddr;
use futures_util::{stream::SplitSink, SinkExt, FutureExt, future};
use tokio::{net::TcpStream, sync::MutexGuard};
use tokio_tungstenite::{WebSocketStream, tungstenite::Message};

use crate::USERS;

/**
 * Send a message to all clients expect the sender.
 */
pub async fn send_all(sender: SocketAddr, content: String) {
    unsafe {
        // Send the message to all users except the one who send it.
        for user in USERS.iter() {
            if user.addr != sender {
                let _ = user.socket.lock().then(|mut socket| async {
                    socket.send(Message::Text(String::clone(&content))).await.expect("Can send message");
                    future::ok::<MutexGuard<SplitSink<WebSocketStream<TcpStream>, Message>>, MutexGuard<SplitSink<WebSocketStream<TcpStream>, Message>>>(socket)
                }).await;
            }
        }
    }
}