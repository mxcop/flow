use std::{net::SocketAddr, sync::Arc};
use colored::*;
use futures_util::{stream::SplitSink};
use serde_json::{Value, json};
use tokio::{net::TcpStream, sync::Mutex};
use tokio_tungstenite::{WebSocketStream, tungstenite::Message};
use uuid::Uuid;

use crate::{info::{self, get_user}, USERS, FluxUser, send::{send_all, send_only}, util::trim_ends};

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

                // Send the new user an update with all online users:
                let mut login_json = json!({
                    "type": "login",
                    "users": []
                });

                let users = login_json["users"].as_array_mut().unwrap();
                for user in USERS.iter() {
                    users.push(json!({
                        "id": user.id,
                        "name": user.name
                    }));
                }

                // Add the new user to the system.
                USERS.push(FluxUser { id: Uuid::new_v4().to_string(), name: trim_ends(json["name"].to_string()), addr, socket });
                
                send_only(addr, login_json.to_string()).await;

                // Send an update to all other users that you've joined:
                let user = get_user(addr);
                let update_json = json!({
                    "type": "join",
                    "user": {
                        "id": user.id,
                        "name": user.name
                    }
                });

                send_all(addr, update_json.to_string()).await;
                
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
        let user = get_user(addr);
        let msg_json = json!({
            "type": "chat",
            "sender": {
                "id": user.id,
                "name": user.name
            },
            "content": json["content"].to_string()
        });

        send_all(addr, msg_json.to_string()).await;

    } else {
        info::user_info(
            addr,
            String::from("Invalid (Message missing content)"),
            Color::Red
        );
    }
}