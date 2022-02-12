use std::{net::SocketAddr, sync::Arc};
use colored::*;
use futures_util::{stream::SplitSink, SinkExt, FutureExt, future};
use serde_json::Value;
use tokio::{net::TcpStream, sync::{Mutex, MutexGuard}};
use tokio_tungstenite::{WebSocketStream, tungstenite::Message};

use crate::{info, USERS, FluxUser};

/**
 * Handle the login message type.
 * This will add the user to the users list.
 */
pub fn login(json: Value, addr: SocketAddr, socket: Arc<Mutex<SplitSink<WebSocketStream<TcpStream>, Message>>>) {
    if json["name"] != Value::Null {
        unsafe {
            // Check if the user isn't already logged in.
            if USERS.iter().all(|user| user.addr != addr) {
                info::info(
                    addr.to_string().white(),
                    format!("@login {}", json["name"]),
                );
                USERS.push(FluxUser { name: json["name"].to_string(), addr, socket });
            } else {
                info::user_info(
                    addr,
                    String::from("Denied (User cannot login twice)"),
                    Color::Red
                );
            }
        }

    } else {
        info::user_info(
            addr,
            String::from("Invalid (Login missing name)"),
            Color::Red
        );
    }
}

/**
 * Handle the chat message type.
 * This will send the recieved message to all connected users.
 */
pub async fn chat(json: Value, addr: SocketAddr) {
    // Check if the user is logged in:
    unsafe {
        if USERS.iter().all(|user| user.addr != addr) {
            info::user_info(
                addr,
                String::from("Invalid (Need to be logged in to chat)"),
                Color::Red
            ); return;
        }
    }

    if json["content"] != Value::Null {
        info::user_info(
            addr,
            json["content"].to_string(),
            Color::Blue
        );
        unsafe {
            for user in USERS.iter() {
                if user.addr != addr {
                    info::info(
                        "SendMessage".white(),
                        addr.to_string()
                    );
                    let _ = user.socket.lock().then(|mut socket| async {
                        socket.send(Message::Text(json["content"].to_string())).await.unwrap();
                        future::ok::<MutexGuard<SplitSink<WebSocketStream<TcpStream>, Message>>, MutexGuard<SplitSink<WebSocketStream<TcpStream>, Message>>>(socket)
                    }).await;
                }
            }
        }

    } else {
        info::user_info(
            addr,
            String::from("Invalid (Message missing content)"),
            Color::Red
        );
    }
}