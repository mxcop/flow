use colored::*;
use futures_util::stream::SplitSink;
use serde_json::{json, Value};
use std::{net::SocketAddr, sync::Arc};
use tokio::{net::TcpStream, sync::Mutex};
use tokio_tungstenite::{tungstenite::Message, WebSocketStream};
use uuid::Uuid;

use crate::{
    info::{self, dispose_offer, get_user, get_user_id, user_exists},
    send::{send_all, send_only},
    utils::{trim_ends},
    FluxUser, Offer, OFFERS, USERS,
};

/**
 * Handle the login message type.
 * This will add the user to the users list.
 */
pub async fn login(
    json: Value,
    addr: SocketAddr,
    socket: Arc<Mutex<SplitSink<WebSocketStream<TcpStream>, Message>>>,
) -> Option<String> {
    if json["name"] != Value::Null {
        unsafe {
            // Check if the user isn't already logged in.
            if USERS.iter().all(|user| user.addr != addr) {
                info::info(addr.to_string().white(), format!("@login {}", json["name"]));

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
                USERS.push(FluxUser {
                    id: Uuid::new_v4().to_string(),
                    name: trim_ends(json["name"].to_string()),
                    addr,
                    socket,
                });

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
                return Some(String::from("User cannot login twice"));
            }
        }
    } else {
        return Some(String::from("Missing name field"));
    }

    None // Succes!
}

/**
 * Handle the chat message type.
 * This will send the recieved message to all connected users.
 */
pub async fn chat(json: Value, addr: SocketAddr) -> Option<String> {
    // Check if the user is logged in:
    if user_exists(addr) == false {
        return Some(String::from("User not authorized"));
    }

    if json["content"] != Value::Null {
        info::user_info(addr, json["content"].to_string(), Color::Blue);

        // Send the message to all other users:
        let user = get_user(addr);
        let msg_json = json!({
            "type": "chat",
            "sender": {
                "id": user.id,
                "name": user.name
            },
            "content": trim_ends(json["content"].to_string())
        });

        send_all(addr, msg_json.to_string()).await;
    } else {
        return Some(String::from("Invalid (Message missing content)"));
    }

    None // Succes!
}

/**
 * Handle the file message type.
 * This will send the recieved file to all connected users.
 */
pub async fn file(json: Value, addr: SocketAddr) -> Option<String> {
    // Check if the user is logged in:
    if user_exists(addr) == false {
        return Some(String::from("User not authorized"));
    }

    if json["content"] != Value::Null && json["name"] != Value::Null {
        info::user_info(addr, json["name"].to_string(), Color::Blue);

        // Send the file to all other users:
        let user = get_user(addr);
        let msg_json = json!({
            "type": "file",
            "sender": {
                "id": user.id,
                "name": user.name
            },
            "name": trim_ends(json["name"].to_string()),
            "content": trim_ends(json["content"].to_string())
        });

        send_all(addr, msg_json.to_string()).await;
    } else {
        return Some(String::from("Invalid (Message missing content or name)"));
    }

    None // Succes!
}

/**
 * Handle the request message type.
 * { type: "request", target: "user_id" }
 * This is called when a user wishes to open a peer connection with another user.
 */
pub async fn request(json: Value, addr: SocketAddr) -> Option<String> {
    // Check if the user is logged in:
    if user_exists(addr) == false {
        return Some(String::from("User not authorized"));
    }

    if json["target"] != Value::Null {
        unsafe {
            // Attempt to find the target user.
            let search_attempt = get_user_id(trim_ends(json["target"].to_string()));

            match search_attempt {
                Some(target) => {
                    info::user_info(
                        addr,
                        format!("Send request to {}", target.name),
                        Color::Magenta,
                    );

                    // Add the offer to the list.
                    let origin = get_user(addr);
                    let offer_id = Uuid::new_v4().to_string();
                    OFFERS.push(Offer {
                        origin: origin.id.clone(),
                        target: target.id.clone(),
                        id: offer_id.clone(),
                    });

                    // Create the offer message:
                    let offer_json = json!({
                        "type": "offer",
                        "origin": origin.id.clone(),
                        "id": offer_id
                    });

                    send_only(target.addr, offer_json.to_string()).await;
                }
                None => return Some(String::from("Request Invalid (target not found)"))
            }
        }
    }

    None // Succes!
}

/**
 * Handle the offer message type.
 * This is called when a user wants to accept or decline an offer.
 */
pub async fn offer(json: Value, addr: SocketAddr) -> Option<String> {
    // Check if the user is logged in:
    if user_exists(addr) == false {
        return Some(String::from("User not authorized"));
    }

    if json["accept"] != Value::Null && json["id"] != Value::Null {
        let accept = json["accept"].as_bool();
        let id = json["id"].as_str();
        match (accept, id) {
            (Some(accept), Some(id)) => {
                unsafe {
                    let offer = OFFERS.iter().find(|&offer| offer.id == id);

                    // See if the user accepted the offer:
                    match offer {
                        Some(offer) => {
                            let target = get_user_id(offer.target.clone());
                            let origin = get_user_id(offer.origin.clone());

                            match (target, origin) {
                                (Some(target), Some(origin)) => {
                                    // Check if the user who sends the response is actually the target.
                                    if target.addr != addr {
                                        return Some(String::from("Access declined"));
                                    }

                                    if accept {
                                        info::user_info(
                                            addr,
                                            format!("Accepted request from {}", origin.name),
                                            Color::Magenta,
                                        );

                                        // Create the confirmation message:
                                        let confirm_json = json!({
                                            "type": "confirm",
                                            "accept": true,
                                            "offer": offer.id.clone()
                                        });

                                        send_only(target.addr, confirm_json.to_string()).await;
                                        send_only(origin.addr, confirm_json.to_string()).await;
                                    } else {
                                        info::user_info(
                                            addr,
                                            format!(
                                                "Declined request from {}",
                                                origin.name
                                            ),
                                            Color::Magenta,
                                        );

                                        // Create the confirmation message:
                                        let confirm_json = json!({
                                            "type": "confirm",
                                            "accept": false,
                                            "offer": offer.id.clone()
                                        });

                                        send_only(target.addr, confirm_json.to_string()).await;
                                        send_only(origin.addr, confirm_json.to_string()).await;

                                        // Remove the offer from the offers.
                                        dispose_offer(offer.id.clone());
                                    }
                                }
                                _ => return Some(String::from("Target or Origin doesn't exist"))
                            }
                        }
                        None => return Some(String::from("Offer not found"))
                    }
                }
            }
            _ => return Some(String::from("Invalid accept or id"))
        }
    } else {
        return Some(String::from("Accept or Id field missing)"));
    }

    None // Succes!
}

/**
 * Handle the session message type.
 * This is send after a p2p offer is accepted, it contains the hole punched port of a user.
 * { type: "session", id: "offer_id", port: "punched_port" }
 */
pub async fn session(json: Value, addr: SocketAddr) -> Option<String> {
    // Check if the user is logged in:
    if user_exists(addr) == false {
        return Some(String::from("User not authorized"));
    }

    if json["offer"] != Value::Null && json["port"] != Value::Null {

        // Get the port and id from the json:
        let port = json["port"].as_u64();
        let offer_id = json["offer"].as_str();

        if let None = offer_id { return Some(String::from("Offer is invalid")); } 
        let offer_id = offer_id.unwrap();
        if let None = port { return Some(String::from("Port is invalid")); } 
        let port = port.unwrap().to_string();

        // Get the offer from offers list:
        let offer;
        unsafe { offer = OFFERS.iter().find(|&offer| offer.id == offer_id); }

        if let None = offer { return Some(String::from("Offer doesn't exist")); }
        let offer = offer.unwrap();

        // Get the target and origin:
        let target = get_user_id(offer.target.clone());
        let origin = get_user_id(offer.origin.clone());

        if let None = target { return Some(String::from("Target doesn't exist")); } 
        let target = target.unwrap();
        if let None = origin { return Some(String::from("Origin doesn't exist")); } 
        let origin = origin.unwrap();

        let peer_addr = format!("{}:{}", addr.ip().to_string(), port);

        // Create the message for the origin:
        let peer_json = json!({
            "type": "peer",
            "addr": peer_addr,
            "offer": offer.id.clone()
        });

        if target.addr == addr {
            send_only(origin.addr, peer_json.to_string()).await;
            dispose_offer(offer.id.clone());
            return None;
        }

        if origin.addr == addr {
            send_only(target.addr, peer_json.to_string()).await;
            dispose_offer(offer.id.clone());
            return None;
        }

        return Some(String::from("Access declined"));
    } else {
        return Some(String::from("Missing id or port"));
    }
}