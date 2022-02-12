use std::{net::SocketAddr, sync::Arc};
use colored::*;
use futures_util::{stream::SplitSink};
use serde_json::Value;
use tokio::{net::TcpStream, sync::Mutex};
use tokio_tungstenite::{WebSocketStream, tungstenite::Message};
use uuid::Uuid;

use crate::{info, USERS, FluxUser, send::send_all};

/**
 * Handle the login message type.
 * This will add the user to the users list.
 */
pub async fn login(json: Value, addr: SocketAddr, socket: Arc<Mutex<SplitSink<WebSocketStream<TcpStream>, Message>>>) {
    if json["name"] != Value::Null {
        unsafe {
            // Check if the user isn't already logged in.
            if USERS.iter().all(|user| user.addr != addr) {
                info::info(
                    addr.to_string().white(),
                    format!("@login {}", json["name"]),
                );
                USERS.push(FluxUser { id: Uuid::new_v4().to_string(), name: json["name"].to_string(), addr, socket });

                // Send an update to all other users that you've joined:
                let index = USERS.iter().position(|user| user.addr == addr).expect("Can find user");
                let user = USERS.get(index).expect("Can read user");
                let usr_update = format!("{{\"type\":\"join\",\"user\":{{\"id\":\"{}\",\"name\":{}}}}}", user.id, user.name);

                send_all(addr, usr_update).await;

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

        // Send the message to all other users:
        unsafe {
            let index = USERS.iter().position(|user| user.addr == addr).expect("Can find user");
            let user = USERS.get(index).expect("Can read user");
            let msg_data = format!("{{\"type\":\"chat\",\"sender\":{{\"id\":\"{}\",\"name\":{}}},\"content\":{}}}", user.id, user.name, json["content"].to_string());

            send_all(addr, msg_data).await;
        }

    } else {
        info::user_info(
            addr,
            String::from("Invalid (Message missing content)"),
            Color::Red
        );
    }
}