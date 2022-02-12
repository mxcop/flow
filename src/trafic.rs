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
            let index = USERS.iter().position(|user| user.addr == addr).expect("Can find user");
            let msg_data = format!("{{\"type\":\"chat\",\"sender\":\"{}\",\"content\":{}}}", USERS.get(index).expect("Can read user").name, json["content"].to_string());

            // Send the message to all users except the one who send it.
            for user in USERS.iter() {
                if user.addr != addr {
                    let _ = user.socket.lock().then(|mut socket| async {
                        socket.send(Message::Text(String::clone(&msg_data))).await.expect("Can send message");
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