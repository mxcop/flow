use std::net::SocketAddr;
use futures_util::{stream::SplitSink, SinkExt, FutureExt, future};
use tokio::{net::TcpStream, sync::MutexGuard};
use tokio_tungstenite::{WebSocketStream, tungstenite::Message};

use crate::{USERS, info::get_user};

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

/**
 * Send a message to only one client.
 */
pub async fn send_only(reciever: SocketAddr, content: String) {
    let user = get_user(reciever);

    // Send the message to the reciever:
    let _ = user.socket.lock().then(|mut socket| async {
        socket.send(Message::Text(String::clone(&content))).await.expect("Can send message");
        future::ok::<MutexGuard<SplitSink<WebSocketStream<TcpStream>, Message>>, MutexGuard<SplitSink<WebSocketStream<TcpStream>, Message>>>(socket)
    }).await;
}